"""Optional Linux Orca/Firefox checks with private buses, a temporary profile and isolated audio.

Requires existing tools and a prepared Orca prefix. Installs nothing. Obtain authorization
for native command control before use. Interaction checks inspect Orca presentation;
page-reading checks require completed Speech Dispatcher messages. Neither calls
speech/presentation functions nor modifies third-party implementation.
"""
import argparse
import array
import ast
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from urllib.parse import urlsplit
import wave
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parents[1]


def configure(argv=None):
    """Prepare a native run only when invoked explicitly through the CLI."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prefix', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'dist/verification/orca')
    parser.add_argument('--base-url', default='http://127.0.0.1:6007')
    parser.add_argument('--cases', type=Path, default=ROOT / 'tests/fixtures/orca-cases.json')
    parser.add_argument('--interactive', action='store_true')
    args = parser.parse_args(argv)
    output = args.output_dir.resolve()
    prefix = args.prefix.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise SystemExit('Use a new, empty output directory to preserve previous evidence')
    try:
        base = urlsplit(args.base_url)
        valid_base = (
            base.scheme == 'http'
            and base.hostname == '127.0.0.1'
            and base.port is not None
            and 0 < base.port <= 65535
            and base.username is None
            and base.password is None
            and base.path in ('', '/')
            and not base.query
            and not base.fragment
        )
    except ValueError:
        valid_base = False
    if not valid_base:
        raise SystemExit('Use an explicit loopback development server')
    args.base_url = args.base_url.rstrip('/')
    for tool in ('firefox', 'xdotool', 'pactl', 'parec', 'dbus-daemon', 'gdbus', 'gsettings', 'import'):
        if not shutil.which(tool):
            raise SystemExit('NOT RUN: missing existing native tool ' + tool + '; nothing installed')
    if not (prefix / 'usr/bin/orca').is_file():
        raise SystemExit('NOT RUN: Orca prefix is unavailable; nothing installed')
    return args, output, prefix

class Marionette:

    def __init__(self, port):
        end = time.monotonic() + 20
        while True:
            try:
                self.sock = socket.create_connection(('127.0.0.1', port), timeout=1)
                break
            except OSError:
                if time.monotonic() > end:
                    raise
                time.sleep(0.1)
        self.sock.settimeout(20)
        self.sequence = 0
        self.receive()
        self.capabilities = self.call('WebDriver:NewSession', {'capabilities': {'alwaysMatch': {'acceptInsecureCerts': False}}})

    def receive(self):
        length = b''
        while (part := self.sock.recv(1)) != b':':
            if not part:
                raise EOFError('Marionette connection closed')
            length += part
        remaining = int(length)
        payload = b''
        while remaining:
            part = self.sock.recv(remaining)
            if not part:
                raise EOFError
            payload += part
            remaining -= len(part)
        return json.loads(payload)

    def call(self, name, parameters=None):
        self.sequence += 1
        data = json.dumps([0, self.sequence, name, parameters or {}]).encode()
        self.sock.sendall(str(len(data)).encode() + b':' + data)
        while True:
            message = self.receive()
            if message[0] == 1 and message[1] == self.sequence:
                if message[2]:
                    raise RuntimeError(message[2])
                return message[3]

    def evaluate(self, script, args=None):
        return self.call('WebDriver:ExecuteScript', {'script': script, 'args': args or [], 'newSandbox': False, 'sandbox': 'default', 'line': 1, 'filename': 'fjord-orca-check'})['value']

def command(*args, env=None, timeout=None):
    result = subprocess.run(
        list(map(str, args)), env=env, text=True, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, timeout=timeout,
    )
    if result.returncode:
        raise RuntimeError(str(args) + ': ' + result.stdout)
    return result.stdout.strip()


def stop_process_group(process, interrupt=False):
    """Stop an owned session even when its original process has already exited."""
    try:
        os.killpg(process.pid, signal.SIGINT if interrupt else signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        pass
    finally:
        # A launcher can exit before its children; waiting for it alone is insufficient.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()


def audio_statistics(path):
    """Record absent/invalid audio as a failed check without hiding startup errors."""
    try:
        with wave.open(str(path), 'rb') as audio:
            samples = array.array('h', audio.readframes(audio.getnframes()))
            return {
                'sampleRate': audio.getframerate(),
                'frames': audio.getnframes(),
                'nonzeroSamples': sum(value != 0 for value in samples),
                'maximum': max(map(abs, samples), default=0),
            }
    except (OSError, EOFError, ValueError, wave.Error) as error:
        return {'nonzeroSamples': 0, 'error': str(error) or type(error).__name__}


def handle_termination(signum, _frame):
    """Let the normal finally blocks release resources on SIGTERM."""
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    raise SystemExit(128 + signum)


def missing_speech(request, presentation, completed):
    source = completed if request.get('readPage') else presentation
    joined = ' '.join(source).casefold()
    exact = {text.casefold() for text in source}
    return {
        'missingSpeech': [text for text in request.get('expectSpeech', [])
                          if text.casefold() not in joined],
        'missingExactSpeech': [text for text in request.get('expectSpeechExact', [])
                               if text.casefold() not in exact],
        'missingCompletedSpeech': [
            text for text in request.get('expectCompletedSpeech', [])
            if not any(text.casefold() in utterance.casefold() for utterance in completed)
        ],
    }


def completed_speech(path, start_offset=0):
    """Match SSML submissions to their queued IDs and actual SSIP END replies.

    A queued, canceled, malformed or partially logged utterance is not completed
    speech. Keep the submission boundary so earlier queued text cannot satisfy a
    later case merely because its playback finishes during that case.
    """
    if not path.exists():
        return []
    records = re.finditer(
        rb'(?ms)^\[[^\r\n]+\] speechd:\s+(\d+):(DATA|REPLY):\|'
        rb'(.*?)(?:\| \((\d+)\)\r?\n|^\|\r?\n)',
        path.read_bytes(),
    )
    receiving = {}
    queued = {}
    completed = []
    for record in records:
        connection = int(record.group(1))
        kind = record.group(2)
        payload = record.group(3)
        if kind == b'DATA':
            # DATA lengths include protocol CRLF; reject truncated log records.
            if record.group(4) is None or int(record.group(4)) != len(payload):
                receiving.pop(connection, None)
                continue
            if payload == b'SPEAK\r\n':
                receiving[connection] = {
                    'offset': record.start(), 'body': b'', 'accepted': False,
                }
                continue
            submission = receiving.get(connection)
            if submission and submission['accepted']:
                submission['body'] += payload
            continue

        submission = receiving.get(connection)
        if payload == b'230 OK RECEIVING DATA\r\n' and submission:
            submission['accepted'] = True
            continue
        queued_id = re.fullmatch(rb'225-(\d+)\r\n225 OK MESSAGE QUEUED\r\n', payload)
        if queued_id and submission:
            receiving.pop(connection, None)
            body = submission['body']
            if not submission['accepted'] or not body.endswith(b'\r\n.\r\n'):
                continue
            # SSIP dot-stuffs text lines beginning with a full stop.
            body = re.sub(rb'(?m)^\.\.', b'.', body[:-5])
            try:
                document = ElementTree.fromstring(body)
            except ElementTree.ParseError:
                continue
            if document.tag.rsplit('}', 1)[-1] != 'speak':
                continue
            text = ' '.join(''.join(document.itertext()).split())
            if text:
                queued[connection, int(queued_id.group(1))] = {
                    'text': text, 'submittedOffset': submission['offset'],
                }
            continue
        event = re.fullmatch(
            rb'(702|703)-(\d+)\r\n\1-(\d+)\r\n\1 (END|CANCELED)\r\n', payload,
        )
        if event:
            message_id = int(event.group(2))
            utterance = queued.pop((connection, message_id), None)
            if event.group(1) == b'702' and event.group(4) == b'END' and utterance:
                if utterance['submittedOffset'] >= start_offset:
                    completed.append({
                        **utterance,
                        'connection': connection,
                        'messageId': message_id,
                        'clientId': int(event.group(3)),
                        'completedOffset': record.start(),
                    })
    return completed


def verify_audio_route(sink, timeout=8):
    """Wait for a native speech stream targeting the owned null sink."""
    deadline = time.monotonic() + timeout
    while True:
        sinks = json.loads(command('pactl', '--format=json', 'list', 'sinks', timeout=3))
        owned = next((item for item in sinks if item['name'] == sink), None)
        inputs = json.loads(command('pactl', '--format=json', 'list', 'sink-inputs', timeout=3))
        targeted = [
            item for item in inputs
            if item.get('properties', {}).get('target.object') == sink
        ]
        if owned and targeted:
            if any(int(item['sink']) != int(owned['index']) for item in targeted):
                raise RuntimeError('Speech stream target and actual sink do not match')
            return {
                'ownedSink': sink,
                'sinkIndex': owned['index'],
                'sinkInputs': [{
                    'index': item['index'], 'sink': item['sink'],
                    'target': item['properties']['target.object'],
                    'processId': item['properties'].get('application.process.id'),
                    'application': item['properties'].get('application.name'),
                } for item in targeted],
            }
        if time.monotonic() >= deadline:
            raise RuntimeError('No native speech stream routed to the owned null sink')
        time.sleep(0.2)


def main():
    processes = []
    module = None
    files = []
    client = None
    recorder = None
    default_sink = command('pactl', 'get-default-sink')
    with tempfile.TemporaryDirectory(prefix='fjord-orca-') as tmp:
        state = Path(tmp)
        (state / 'runtime').mkdir(mode=448)
        (OUT / 'state-path.txt').write_text(tmp + '\n')
        env = dict(os.environ, PATH=str(PREFIX / 'usr/bin') + ':' + os.environ['PATH'], LD_LIBRARY_PATH=str(PREFIX / 'usr/lib'), PYTHONPATH=str(PREFIX / 'usr/lib/python3.14/site-packages'), PYTHONDONTWRITEBYTECODE='1', GI_TYPELIB_PATH=str(PREFIX / 'usr/lib/girepository-1.0'), XDG_DATA_DIRS=str(PREFIX / 'usr/share') + ':/usr/local/share:/usr/share', XDG_CONFIG_HOME=str(state / 'config'), XDG_DATA_HOME=str(state / 'data'), XDG_CACHE_HOME=str(state / 'cache'), GSETTINGS_BACKEND='keyfile', GSETTINGS_SCHEMA_DIR=str(PREFIX / 'usr/share/glib-2.0/schemas'), ESPEAK_DATA_PATH=str(PREFIX / 'usr/share/espeak-ng-data'))

        def spawn(name, args, extra=None):
            log = (OUT / (name + '.log')).open('w')
            files.append(log)
            p = subprocess.Popen(list(map(str, args)), env={**env, **(extra or {})}, stdout=log, stderr=log, start_new_session=True)
            processes.append(p)
            return p
        try:
            env['XDG_RUNTIME_DIR'] = str(state / 'runtime')
            env['PULSE_SERVER'] = 'unix:' + os.environ['XDG_RUNTIME_DIR'] + '/pulse/native'
            env['SPEECHD_PLUGIN_DIR'] = str(PREFIX / 'usr/lib/speech-dispatcher')
            env['GTK_USE_PORTAL'] = '0'
            env['GDK_BACKEND'] = 'x11'
            env['QT_QPA_PLATFORM'] = 'xcb'
            bus_config = state / 'dbus.conf'
            bus_config.write_text('<busconfig><type>session</type><listen>unix:path=' + str(state / 'bus') + '</listen><auth>EXTERNAL</auth><policy context="default"><allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/></policy></busconfig>')
            bus = spawn('private-bus', ['dbus-daemon', '--nofork', '--config-file', bus_config])
            end = time.monotonic() + 5
            while not (state / 'bus').exists():
                if bus.poll() is not None or time.monotonic() > end:
                    raise RuntimeError('Private DBus failed')
                time.sleep(0.05)
            address = 'unix:path=' + str(state / 'bus')
            assert address != os.environ['DBUS_SESSION_BUS_ADDRESS']
            env['DBUS_SESSION_BUS_ADDRESS'] = address
            launcher = spawn('accessibility-launcher', ['/usr/lib/at-spi-bus-launcher', '--launch-immediately'], {'ATSPI_DBUS_IMPLEMENTATION': 'dbus-daemon', 'DISPLAY': ''})
            end = time.monotonic() + 5
            while True:
                try:
                    a11y = ast.literal_eval(command('gdbus', 'call', '--session', '--dest', 'org.a11y.Bus', '--object-path', '/org/a11y/bus', '--method', 'org.a11y.Bus.GetAddress', env=env))[0]
                    break
                except RuntimeError:
                    if launcher.poll() is not None or time.monotonic() > end:
                        raise
                    time.sleep(0.1)
            env['AT_SPI_BUS_ADDRESS'] = a11y
            sink = 'fjord_orca_' + str(os.getpid())
            module = command('pactl', 'load-module', 'module-null-sink', 'sink_name=' + sink, 'sink_properties=device.description=Fjord_Orca_Test')
            assert command('pactl', 'get-default-sink') == default_sink
            env['PULSE_SINK'] = sink
            # Native PipeWire avoids Pulse compatibility drain stalls on this target.
            env['PIPEWIRE_RUNTIME_DIR'] = os.environ['XDG_RUNTIME_DIR']
            env['PIPEWIRE_NODE'] = sink
            env['PIPEWIRE_PROPS'] = '{ node.dont-fallback = true }'
            config = state / 'speechd'
            config.mkdir()
            (config / 'modules').mkdir()
            (config / 'modules/espeak-ng.conf').write_text((PREFIX / 'etc/speech-dispatcher/modules/espeak-ng.conf').read_text())
            sock = state / 'speechd.sock'
            (config / 'speechd.conf').write_text(f'CommunicationMethod "unix_socket"\nSocketPath "{sock}"\nLogLevel 1\nCustomLogFile "protocol" "{OUT / 'speechd-protocol.log'}"\nDefaultLanguage "en-US"\nAudioOutputMethod "pipewire"\nAddModule "espeak-ng" "sd_espeak-ng" "espeak-ng.conf"\nDefaultModule espeak-ng\n')
            (OUT / 'speechd').mkdir(exist_ok=True)
            speech = spawn('speechd', [PREFIX / 'usr/bin/speech-dispatcher', '-s', '-C', config, '-m', PREFIX / 'usr/lib/speech-dispatcher/speech-dispatcher-modules', '-S', sock, '-P', state / 'speechd.pid', '-L', OUT / 'speechd', '-t', '0'])
            end = time.monotonic() + 10
            while not sock.exists():
                if speech.poll() is not None or time.monotonic() > end:
                    raise RuntimeError('Speech Dispatcher failed; see speechd.log')
                time.sleep(0.1)
            env['SPEECHD_ADDRESS'] = 'unix_socket:' + str(sock)
            for voice in ('default', 'system', 'uppercase', 'hyperlink'):
                command('gsettings', 'set', 'org.gnome.Orca.Voice:/org/gnome/orca/default/voice-sets/primary/' + voice + '/', 'rate', '90', env=env)
            recorder = spawn('recording', ['parec', '--device=' + sink + '.monitor', '--format=s16le', '--rate=22050', '--channels=1', '--file-format=wav', OUT / 'speech.wav'])
            orca = spawn('orca', [PREFIX / 'usr/bin/orca', '--speech-system', 'speechdispatcherfactory', '--debug-file', OUT / 'orca-debug.log'])
            profile = state / 'firefox'
            profile.mkdir()
            with socket.socket() as port_probe:
                port_probe.bind(('127.0.0.1', 0))
                port = port_probe.getsockname()[1]
            prefs = {'marionette.port': port, 'marionette.enabled': True, 'browser.shell.checkDefaultBrowser': False, 'browser.startup.homepage_override.mstone': 'ignore', 'browser.startup.firstrunSkipsHomepage': True, 'browser.aboutwelcome.enabled': False, 'browser.startup.page': 0, 'datareporting.policy.dataSubmissionEnabled': False, 'toolkit.telemetry.enabled': False, 'accessibility.force_disabled': -1, 'widget.use-xdg-desktop-portal.file-picker': 0, 'widget.use-xdg-desktop-portal.mime-handler': 0, 'widget.use-xdg-desktop-portal.settings': 0, 'widget.use-xdg-desktop-portal.open-uri': 0}
            (profile / 'user.js').write_text('\n'.join(('user_pref(' + json.dumps(key) + ',' + json.dumps(value) + ');' for key, value in prefs.items())) + '\n')
            firefox = spawn('firefox', ['/usr/bin/firefox', '--new-instance', '--profile', profile, '--marionette', ARGS.base_url + '/iframe.html?id=complete-library-disclosures--night&viewMode=story'], {'MOZ_ENABLE_WAYLAND': '0', 'NO_AT_BRIDGE': '0'})
            client = Marionette(port)
            end = time.monotonic() + 15
            window = None
            while not window:
                result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--pid', str(firefox.pid)], text=True, capture_output=True)
                if result.returncode == 0 and result.stdout.strip():
                    window = result.stdout.strip().splitlines()[0]
                if time.monotonic() > end:
                    raise RuntimeError('Owned Firefox window not found')
                time.sleep(0.1)
            command('xdotool', 'windowactivate', '--sync', window)
            time.sleep(2)
            metadata = {'orcaVersion': (PREFIX / 'usr/lib/python3.14/site-packages/orca/orca_platform.py').read_text().split('version     = ')[1].splitlines()[0].strip(chr(34))}
            if orca.poll() is not None:
                raise RuntimeError('Orca did not remain running; inspect orca.log')
            routing = verify_audio_route(sink)
            (OUT / 'audio-routing.json').write_text(json.dumps(routing, indent=2) + '\n')
            print(json.dumps({
                'ready': True, 'firefoxPid': firefox.pid, 'window': window,
                'orcaPid': orca.pid, 'orcaRunning': True,
                'browser': client.capabilities, 'privateAccessibilityBus': a11y,
                'audioRouting': routing,
                'defaultSinkUnchanged': command('pactl', 'get-default-sink') == default_sink,
            }), flush=True)
            results = []
            (OUT / 'results.json').write_text('[]\n')
            debug_path = OUT / 'orca-debug.log'
            protocol_path = OUT / 'speechd-protocol.log'

            def completed_utterances(offset):
                return [item['text'] for item in completed_speech(protocol_path, offset)]

            def utterances(offset):
                text = debug_path.read_bytes()[offset:].decode(errors='replace')
                return [ast.literal_eval(match.group(1)) for match in re.finditer('SPEECH OUTPUT: (\'(?:\\\\.|[^\'])*\'|\\"(?:\\\\.|[^\\"])*\\")', text)]

            def execute(request):
                action = request['action']
                if action == 'evaluate':
                    return client.evaluate(request['script'], request.get('args'))
                if action == 'assert':
                    result = client.evaluate(request['script'])
                    if result is not True:
                        raise AssertionError(request.get('label', 'DOM state') + ': ' + str(result))
                    return True
                if action == 'focus':
                    value = client.evaluate('const el=document.querySelector(arguments[0]);if(!el)throw new Error("Missing focus target");document.activeElement.blur();el.focus();return document.activeElement===el;', [request['selector']])
                    if not value:
                        raise AssertionError('Target did not receive focus: ' + request['selector'])
                    time.sleep(0.7)
                    return value
                if action == 'goto':
                    offset = debug_path.stat().st_size
                    client.call('WebDriver:Navigate', {'url': ARGS.base_url + request['path']})
                    deadline = time.monotonic()+12
                    while not any('Finished loading' in item for item in utterances(offset)):
                        if time.monotonic()>deadline:raise RuntimeError('Orca did not finish presenting the new document')
                        time.sleep(.1)
                    time.sleep(.4)
                    if not request.get('allowReading'):
                        execute({'action':'key','keys':['Control_L'],'settle':.4})
                    return client.evaluate('return document.title')
                if action == 'mode':
                    desired = request['value'].capitalize()+' mode'
                    for _ in range(2):
                        offset = debug_path.stat().st_size
                        execute({'action':'key','keys':['Insert+a'],'settle':.5})
                        spoken = utterances(offset)
                        if desired in spoken:return desired
                    raise RuntimeError('Orca did not enter '+desired)
                if action == 'key':
                    if command('xdotool', 'getactivewindow') != window:
                        raise RuntimeError('Owned Firefox lost focus; input stopped')
                    for _ in range(min(150, request.get('repeat', 1))):
                        command('xdotool', 'key', '--clearmodifiers', '--window', window, *request['keys'])
                        time.sleep(request.get('settle', 0.5))
                    return client.evaluate('return {tag:document.activeElement.tagName,id:document.activeElement.id,role:document.activeElement.getAttribute("role")};')
                if action == 'type':
                    target = client.call('WebDriver:FindElement', {'using': 'css selector', 'value': request['selector']})['value']
                    identifier = target.get('element-6066-11e4-a52e-4f735466cecf', target.get('ELEMENT'))
                    client.call('WebDriver:ElementSendKeys', {'id': identifier, 'text': request['text'], 'value': list(request['text'])})
                    time.sleep(0.4)
                    return True
                if action == 'touch':
                    point = client.evaluate('const el=document.querySelector(arguments[0]);if(!el)throw new Error("Missing touch target");el.scrollIntoView({block:"center"});const r=el.getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};', [request['selector']])
                    actions = [
                        {'type': 'pointerMove', 'duration': 0, 'origin': 'viewport', **point},
                        {'type': 'pointerDown', 'button': 0},
                        {'type': 'pause', 'duration': 80},
                    ]
                    if request.get('dy'):
                        actions.append({'type': 'pointerMove', 'duration': 250, 'origin': 'viewport', 'x': point['x'], 'y': point['y'] + request['dy']})
                    actions.append({'type': 'pointerUp', 'button': 0})
                    try:
                        client.call('WebDriver:PerformActions', {'actions': [{'type': 'pointer', 'id': 'fjord-touch', 'parameters': {'pointerType': 'touch'}, 'actions': actions}]})
                    finally:
                        client.call('WebDriver:ReleaseActions')
                    time.sleep(0.5)
                    return True
                if action == 'capture':
                    name = Path(request['filename'])
                    if name.name != str(name) or name.suffix != '.png':
                        raise ValueError('Capture needs a PNG filename')
                    command('import', '-window', window, OUT / name)
                    return str(name)
                if action == 'wait':
                    time.sleep(min(15, request.get('seconds', 1)))
                    return 'settled'
                raise ValueError('Unknown action ' + action)
            requests = (json.loads(line) for line in sys.stdin) if ARGS.interactive else json.loads(ARGS.cases.read_text())
            for request in requests:
                if request['action'] == 'quit':
                    break
                try:
                    if request['action'] == 'case':
                        if orca.poll() is not None:
                            raise RuntimeError('Orca exited')
                        if command('xdotool', 'getwindowpid', window) != str(firefox.pid):
                            raise RuntimeError('Owned window PID changed')
                        command('xdotool', 'windowactivate', '--sync', window)
                        offset = debug_path.stat().st_size
                        protocol_offset = protocol_path.stat().st_size
                        for operation in request['operations']:
                            if request.get('readPage') and operation['action']=='goto':
                                operation={**operation,'allowReading':True}
                            execute(operation)
                        if request.get('readPage'):
                            deadline=time.monotonic()+120
                            while any(missing_speech(
                                request, [], completed_utterances(protocol_offset),
                            ).values()):
                                if time.monotonic()>deadline:break
                                time.sleep(.2)
                            execute({'action':'key','keys':['Control_L'],'settle':.4})
                        time.sleep(0.5)
                        speech = utterances(offset)
                        finished_speech = completed_utterances(protocol_offset)
                        missing = missing_speech(request, speech, finished_speech)
                        result = {
                            'case': request['name'], 'pass': not any(missing.values()),
                            **missing, 'speech': speech,
                            'completedSpeech': finished_speech,
                            'speechSource': 'completedSpeech' if request.get('readPage') else 'orcaPresentation',
                        }
                        results.append(result)
                        (OUT / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
                    else:
                        result = execute(request)
                    print(json.dumps({'action': request['action'], 'result': result}), flush=True)
                except Exception as error:
                    result = {'case': request.get('name', request['action']), 'pass': False, 'error': str(error)}
                    results.append(result)
                    (OUT / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
                    print(json.dumps(result), flush=True)
            finished_speech = completed_speech(protocol_path)
            (OUT / 'completed-speech.json').write_text(json.dumps(finished_speech, indent=2) + '\n')
            (OUT / 'session.json').write_text(json.dumps({
                'browser': client.capabilities, 'orca': metadata['orcaVersion'],
                'privateSessionBus': address, 'privateAccessibilityBus': a11y,
                'speechFactory': 'speechdispatcherfactory', 'audioBackend': 'pipewire',
                'completedSpeechCount': len(finished_speech), 'cases': len(results),
                'passed': sum(result['pass'] for result in results),
            }, indent=2) + '\n')
        finally:
            signal.signal(signal.SIGTERM, signal.SIG_IGN)
            original_error = sys.exception()
            cleanup_errors = []
            if client:
                try:
                    client.call('Marionette:Quit', {'flags': ['eForceQuit']})
                except Exception:
                    pass
            for process in reversed(processes):
                try:
                    stop_process_group(process, interrupt=process is recorder)
                except Exception as error:
                    cleanup_errors.append(f'Process group {process.pid}: {error}')
            for file in files:
                try:
                    file.close()
                except OSError as error:
                    cleanup_errors.append(f'Log file: {error}')
            if module:
                try:
                    command('pactl', 'unload-module', module)
                except RuntimeError as error:
                    cleanup_errors.append(f'Audio sink: {error}')
            stats = audio_statistics(OUT / 'speech.wav')
            (OUT / 'audio.json').write_text(json.dumps(stats, indent=2) + '\n')
            try:
                sink_unchanged = command('pactl', 'get-default-sink') == default_sink
                if not sink_unchanged:
                    cleanup_errors.append('Default audio sink changed during the check')
            except RuntimeError as error:
                sink_unchanged = None
                cleanup_errors.append(f'Default audio sink check: {error}')
            cleanup = {
                'cleanup': not cleanup_errors,
                'defaultSinkUnchanged': sink_unchanged,
                'errors': cleanup_errors,
            }
            (OUT / 'cleanup.json').write_text(json.dumps(cleanup, indent=2) + '\n')
            print(json.dumps(cleanup), flush=True)
            if cleanup_errors and original_error is None:
                raise RuntimeError('Native cleanup failed; inspect cleanup.json')
if __name__ == '__main__':
    signal.signal(signal.SIGTERM, handle_termination)
    ARGS, OUT, PREFIX = configure()
    main()
    results = json.loads((OUT / 'results.json').read_text())
    audio = json.loads((OUT / 'audio.json').read_text())
    session = json.loads((OUT / 'session.json').read_text())
    if (not results or any(not case['pass'] for case in results)
            or not audio['nonzeroSamples'] or not session['completedSpeechCount']):
        raise SystemExit('FAIL screen-reader cases or synthesized audio; inspect results.json and logs')
    print(f'PASS {len(results)} native Orca cases; {session["completedSpeechCount"]} completed speech messages and non-silent isolated audio')
