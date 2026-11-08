"""Optional native Firefox control regressions; uses existing tools, installs nothing."""
import argparse
import base64
from contextlib import ExitStack
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import tempfile
import threading
import time

from check_orca import Marionette, handle_termination, stop_process_group


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url', help='Override the built Storybook fixture URL')
    parser.add_argument('--output-dir', type=Path)
    args = parser.parse_args()
    signal.signal(signal.SIGTERM, handle_termination)
    firefox = shutil.which('firefox')
    if not firefox:
        raise SystemExit('NOT RUN: Firefox is unavailable; nothing installed')
    evidence = Path('dist/verification')
    evidence.mkdir(parents=True, exist_ok=True)
    output = args.output_dir or Path(tempfile.mkdtemp(prefix='controls-firefox-', dir=evidence))
    if output.exists() and any(output.iterdir()):
        raise SystemExit('Use a new empty output directory to preserve earlier evidence')
    output.mkdir(parents=True, exist_ok=True)
    results = []
    with ExitStack() as stack:
        if args.url:
            url = args.url
        else:
            catalogue = Path('dist/storybook')
            if not (catalogue / 'iframe.html').is_file():
                raise SystemExit('NOT RUN: build Storybook first with mise run storybook:build')

            class QuietHandler(SimpleHTTPRequestHandler):
                def log_message(self, *_args):
                    pass

            server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(catalogue)))
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            stack.callback(server.server_close)
            stack.callback(server.shutdown)
            url = f'http://127.0.0.1:{server.server_port}/iframe.html?id=verification-control-regressions--stepped-values-and-long-lists&viewMode=story'
        profile = stack.enter_context(tempfile.TemporaryDirectory(prefix='fjord-controls-'))
        with socket.socket() as reservation:
            reservation.bind(('127.0.0.1', 0))
            port = reservation.getsockname()[1]
        prefs = {'marionette.port': port, 'browser.shell.checkDefaultBrowser': False,
                 'browser.startup.page': 0, 'browser.aboutwelcome.enabled': False,
                 'datareporting.policy.dataSubmissionEnabled': False, 'toolkit.telemetry.enabled': False}
        Path(profile, 'user.js').write_text('\n'.join(f'user_pref({json.dumps(k)}, {json.dumps(v)});' for k, v in prefs.items()))
        log = stack.enter_context((output / 'firefox.log').open('w'))
        process = subprocess.Popen([firefox, '--headless', '--no-remote', '--profile', profile,
                                    '--marionette', 'about:blank'], stdout=log, stderr=subprocess.STDOUT,
                                   start_new_session=True, env={**os.environ, 'MOZ_HEADLESS_WIDTH': '1200', 'MOZ_HEADLESS_HEIGHT': '1100'})
        stack.callback(stop_process_group, process)
        browser = Marionette(port)
        stack.callback(browser.sock.close)
        browser.call('WebDriver:SetWindowRect', {'width': 1200, 'height': 1100})
        browser.call('WebDriver:Navigate', {'url': url})

        def evaluate(script):
            return browser.evaluate(script)

        def wait(expression):
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if evaluate('return ' + expression):
                    return
                time.sleep(0.05)
            raise AssertionError('Timed out: ' + expression)

        wait("!!document.querySelector('#regression-slider') && document.fonts.status === 'loaded'")

        def click(selector):
            element = browser.call('WebDriver:FindElement', {'using': 'css selector', 'value': selector})
            browser.call('WebDriver:ElementClick', {'id': element['value']['element-6066-11e4-a52e-4f735466cecf']})

        def focus(selector):
            evaluate(f'document.querySelector({json.dumps(selector)}).focus(); return true')

        def keys(*values):
            actions = []
            for value in values:
                actions.extend([{'type': 'keyDown', 'value': value}, {'type': 'keyUp', 'value': value}])
            browser.call('WebDriver:PerformActions', {'actions': [{'type': 'key', 'id': 'keyboard', 'actions': actions}]})

        def check(name, probe):
            try:
                data = probe()
                if not data.pop('passed'):
                    raise AssertionError(json.dumps(data))
                results.append({'name': name, 'passed': True, 'observed': data})
            except Exception as error:
                results.append({'name': name, 'passed': False, 'error': str(error)})
            (output / 'results.json').write_text(json.dumps({'capabilities': browser.capabilities, 'cases': results}, indent=2))
            print(f"{'PASS' if results[-1]['passed'] else 'FAIL'} {name}", flush=True)

        def slider(expected):
            return evaluate(f'''const range = document.querySelector('#regression-slider');
                const exact = document.querySelector('#regression-slider-exact');
                const bound = Number(document.querySelector('#slider-bound').textContent);
                const displayed = Number(document.querySelector('#regression-slider-value').textContent);
                const submitted = Number(new FormData(document.querySelector('#slider-form')).get('volume'));
                return {{range: range.valueAsNumber, exact: exact.valueAsNumber, bound, displayed, submitted,
                    valid: exact.validity.valid, passed: [range.valueAsNumber, exact.valueAsNumber, bound, displayed, submitted].every(v => v === {expected}) && exact.validity.valid}};''')

        check('slider initial off-step value', lambda: slider(20))
        click('#slider-same-value')
        check('slider changed bounds with unchanged normalized value', lambda: slider(20))
        click('#slider-reset')
        click('#slider-external')
        check('slider externally supplied off-step value', lambda: slider(40))
        focus('#regression-slider')
        keys('\ue014')
        check('slider native keyboard edit', lambda: slider(50))
        evaluate("const e=document.querySelector('#regression-slider-exact'); e.value='25'; e.dispatchEvent(new Event('change', {bubbles:true})); return true")
        check('slider exact input positive tie', lambda: slider(30))
        evaluate("const e=document.querySelector('#regression-slider-exact'); e.value='26'; e.dispatchEvent(new Event('change', {bubbles:true})); return true")
        check('slider exact edit returning to existing normalized value', lambda: slider(30))
        evaluate("const e=document.querySelector('#regression-slider-exact'); e.value='150'; e.dispatchEvent(new Event('change', {bubbles:true})); return true")
        check('slider exact edit above maximum', lambda: slider(100))
        evaluate("const e=document.querySelector('#regression-slider-exact'); e.value='-15'; e.dispatchEvent(new Event('change', {bubbles:true})); return true")
        check('slider exact edit below minimum', lambda: slider(0))
        focus('#regression-slider-exact')
        evaluate("document.querySelector('#regression-slider-exact').select(); return true")
        keys('2', '0')
        check('slider precise entry retains intermediate digits', lambda: evaluate("return {passed:document.querySelector('#regression-slider-exact').value === '20' && document.querySelector('#slider-bound').textContent === '0'}"))
        keys('\ue004')
        check('slider precise entry commits on Tab', lambda: slider(20))
        focus('#regression-slider-exact')
        evaluate("document.querySelector('#regression-slider-exact').select(); return true")
        keys('3', '5', '\ue007')
        check('slider precise entry commits on Enter', lambda: slider(40))
        check('slider Enter submission uses committed normalized value', lambda: evaluate("const submitted=document.querySelector('#slider-submitted').textContent; return {submitted, passed:submitted === '40'}"))
        click('#slider-offset')
        check('slider offset minimum and unaligned maximum', lambda: slider(19))
        click('#slider-decimal')
        check('slider fractional positive tie', lambda: slider(0.4))
        click('#slider-step')
        check('slider externally changed step', lambda: slider(0.5))
        click('#slider-min')
        check('slider externally changed minimum', lambda: slider(0.6))

        def selection(expected, valid, submissions):
            return evaluate(f'''const form=document.querySelector('#select-form'), proxy=form.querySelector('select');
                const value=proxy.value, submitted=new FormData(form).get('destination');
                const count=Number(document.querySelector('#submissions').textContent);
                return {{value, submitted, count, valid: proxy.validity.valid,
                  passed: value === {json.dumps(expected)} && submitted === {json.dumps(expected)} && proxy.validity.valid === {str(valid).lower()} && count === {submissions}}};''')

        check('required Select initially disabled option', lambda: selection('', False, 0))
        click('#select-submit')
        check('required Select blocks submission and focuses trigger', lambda: evaluate("return {passed: document.activeElement.id === 'regression-select' && document.querySelector('#regression-select').getAttribute('aria-invalid') === 'true' && document.querySelector('#select-form [role=alert]').textContent.includes('Choose an available option') && document.querySelector('#submissions').textContent === '0'}"))
        click('#select-enable')
        click('#select-submit')
        check('required Select enabled option submits', lambda: selection('item-29', True, 1))
        click('#select-disable')
        click('#select-submit')
        check('required Select dynamically disabled option blocks submission', lambda: selection('', False, 1))
        click('#select-restore')
        click('#select-remove')
        click('#select-submit')
        check('required Select removed option blocks submission', lambda: selection('', False, 1))
        click('#select-restore')

        def visible_active(control, expected, page_y):
            return evaluate(f'''const control=document.querySelector({json.dumps(control)});
                const active=document.getElementById(control.getAttribute('aria-activedescendant'));
                const list=document.getElementById(control.getAttribute('aria-controls'));
                const box=list.getBoundingClientRect(), item=active?.getBoundingClientRect();
                const visible=!!item && item.top >= box.top + list.clientTop - 1 && item.bottom <= box.top + list.clientTop + list.clientHeight + 1;
                return {{active: active?.id, visible, scrollTop:list.scrollTop, focus:document.activeElement.id, pageY:scrollY,
                    passed: visible && active.id === control.id + '-option-' + {expected} && document.activeElement === control && scrollY === {page_y}}};''')

        for control in ['#regression-combo', '#regression-select']:
            focus(control)
            keys('\ue015')
            page_y = evaluate('return scrollY')
            check(control + ' first option', lambda: visible_active(control, 0, page_y))
            keys('\ue010')
            check(control + ' End reveals final option', lambda: visible_active(control, 29, page_y))
            keys('\ue011')
            check(control + ' Home reveals first option', lambda: visible_active(control, 0, page_y))
            keys('\ue013')
            check(control + ' ArrowUp wraps visibly', lambda: visible_active(control, 29, page_y))
            keys('\ue015')
            check(control + ' ArrowDown wraps visibly', lambda: visible_active(control, 0, page_y))
            keys(*(['\ue015'] * 20))
            check(control + ' repeated arrows skip disabled option and scroll', lambda: visible_active(control, 21, page_y))
            keys('\ue00c')
            check(control + ' Escape preserves focus', lambda: evaluate(f"const e=document.querySelector({json.dumps(control)}); return {{passed:document.activeElement===e && e.getAttribute('aria-expanded')==='false' && !e.hasAttribute('aria-activedescendant')}}"))
        focus('#regression-select')
        page_y = evaluate('return scrollY')
        keys('z')
        check('Select typeahead reveals matching option', lambda: visible_active('#regression-select', 29, page_y))
        screenshot = browser.call('WebDriver:TakeScreenshot', {'full': False})['value']
        (output / 'select-typeahead.png').write_bytes(base64.b64decode(screenshot, validate=True))
        keys('\ue007')
        click('#select-submit')
        check('Select visible choice commits and submits', lambda: selection('item-29', True, 2))
        click('#regression-select')
        page_y = evaluate('return scrollY')
        check('Select reopening reveals selected option', lambda: visible_active('#regression-select', 29, page_y))
        browser.call('Marionette:Quit', {'flags': ['eForceQuit']})
    print(f"{sum(case['passed'] for case in results)}/{len(results)} checks passed; evidence: {output}")
    return 0 if all(case['passed'] for case in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
