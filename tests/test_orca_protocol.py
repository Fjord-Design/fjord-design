"""Verify that native speech evidence requires a complete SSIP END event."""

import importlib.util
from pathlib import Path
import signal
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_orca.py"
SPEC = importlib.util.spec_from_file_location("orca_check", SCRIPT)
ORCA = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORCA)


def record(connection, kind, payload):
    """Use Speech Dispatcher 0.12.1's raw protocol-log framing and byte count."""
    header = f"[Sun Nov  8 12:38:31 2026 : 606477] speechd:     {connection}:{kind}:|".encode()
    ending = f"| ({len(payload)})\n".encode() if kind == "DATA" else b"|\n"
    return header + payload + ending


def submission(connection, message_id, xml, fragments=None):
    chunks = fragments if fragments is not None else [xml.encode() + b"\r\n"]
    return b"".join([
        record(connection, "DATA", b"SPEAK\r\n"),
        record(connection, "REPLY", b"230 OK RECEIVING DATA\r\n"),
        *(record(connection, "DATA", chunk) for chunk in chunks),
        record(connection, "DATA", b".\r\n"),
        record(connection, "REPLY", f"225-{message_id}\r\n225 OK MESSAGE QUEUED\r\n".encode()),
    ])


def event(connection, message_id, code=702, name="END"):
    return record(connection, "REPLY", f"{code}-{message_id}\r\n{code}-1\r\n{code} {name}\r\n".encode())


class OrcaProtocolTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.log = Path(temporary.name) / "speechd-protocol.log"

    def parse(self, data, start_offset=0):
        self.log.write_bytes(data)
        return ORCA.completed_speech(self.log, start_offset)

    def test_queued_or_started_text_is_not_completed(self):
        queued = submission(26, 1, "<speak>Sort ascending</speak>")
        self.assertEqual(self.parse(queued), [])
        self.assertEqual(self.parse(queued + event(26, 1, 701, "BEGIN")), [])
        complete = self.parse(queued + event(26, 1))
        self.assertEqual([item["text"] for item in complete], ["Sort ascending"])

    def test_canceled_message_cannot_pass_as_completed(self):
        data = submission(26, 1, "<speak>Validation failed</speak>")
        data += event(26, 1, 703, "CANCELED")
        self.assertEqual(self.parse(data), [])
        self.assertEqual(self.parse(data + event(26, 1)), [])

    def test_partial_end_record_is_not_completed(self):
        data = submission(26, 1, "<speak>Ready</speak>")
        ending = event(26, 1)
        self.assertEqual(self.parse(data + ending[:-2]), [])
        self.assertEqual(self.parse(data + ending)[0]["text"], "Ready")

    def test_case_boundary_uses_submission_time(self):
        earlier = submission(26, 1, "<speak>Earlier case</speak>")
        current = submission(26, 2, "<speak>Current case</speak>")
        data = earlier + current + event(26, 1) + event(26, 2)
        self.assertEqual(
            [item["text"] for item in self.parse(data, len(earlier))],
            ["Current case"],
        )

    def test_fragmented_ssml_is_decoded_after_completion(self):
        xml = '<speak xmlns="http://www.w3.org/2001/10/synthesis">A &amp; <mark name="x"/>café</speak>'
        encoded = xml.encode() + b"\r\n"
        split = encoded.index(b"\xc3") + 1
        fragments = [encoded[:9], encoded[9:split], encoded[split:]]
        data = submission(26, 1, xml, fragments) + event(26, 1)
        self.assertEqual(self.parse(data)[0]["text"], "A & café")

    def test_connections_with_same_message_id_are_distinct(self):
        data = submission(26, 1, "<speak>First connection</speak>")
        data += submission(27, 1, "<speak>Second connection</speak>")
        data += event(27, 1) + event(26, 1)
        self.assertEqual(
            [(item["connection"], item["text"]) for item in self.parse(data)],
            [(27, "Second connection"), (26, "First connection")],
        )

    def test_malformed_xml_or_truncated_data_is_not_completed(self):
        malformed = submission(26, 1, "<speak>Missing end tag")
        self.assertEqual(self.parse(malformed + event(26, 1)), [])
        truncated = submission(26, 2, "<speak>Valid text</speak>")
        truncated = truncated.replace(b"Valid text", b"Shorter")
        self.assertEqual(self.parse(truncated + event(26, 2)), [])

    def test_import_does_not_parse_arguments_touch_output_or_run_tools(self):
        spec = importlib.util.spec_from_file_location("orca_import_probe", SCRIPT)
        module = importlib.util.module_from_spec(spec)
        with (
            patch("argparse.ArgumentParser.parse_args", side_effect=AssertionError("CLI parsing during import")),
            patch("pathlib.Path.mkdir", side_effect=AssertionError("Output setup during import")),
            patch("shutil.which", side_effect=AssertionError("Native tool discovery during import")),
            patch("subprocess.run", side_effect=AssertionError("Native command during import")),
            patch("subprocess.Popen", side_effect=AssertionError("Native process during import")),
            patch("signal.signal", side_effect=AssertionError("Signal handler changed during import")),
        ):
            spec.loader.exec_module(module)
        self.assertTrue(callable(module.completed_speech))

    def test_sigterm_unwinds_cleanup_and_ignores_repeated_signals(self):
        cleaned_up = False
        with patch.object(ORCA.signal, "signal") as register:
            with self.assertRaises(SystemExit) as raised:
                try:
                    ORCA.handle_termination(signal.SIGTERM, None)
                finally:
                    cleaned_up = True
        self.assertEqual(raised.exception.code, 128 + signal.SIGTERM)
        register.assert_called_once_with(signal.SIGTERM, signal.SIG_IGN)
        self.assertTrue(cleaned_up)

    def test_exact_state_does_not_accept_opposite_state(self):
        request = {"expectSpeechExact": ["pressed"]}
        self.assertEqual(
            ORCA.missing_speech(request, ["not pressed"], [])["missingExactSpeech"],
            ["pressed"],
        )
        self.assertFalse(any(ORCA.missing_speech(request, ["pressed"], []).values()))

    def test_completed_expectation_cannot_use_presentation_or_join_messages(self):
        request = {"expectCompletedSpeech": ["Sorted by Count, ascending."]}
        intended = ["Sorted by Count, ascending."]
        for completed in ([], ["Sorted by Count,", "ascending."]):
            with self.subTest(completed=completed):
                self.assertEqual(
                    ORCA.missing_speech(request, intended, completed)["missingCompletedSpeech"],
                    request["expectCompletedSpeech"],
                )
        completed = ["3 rows; 0 selected. Sorted by Count, ascending."]
        self.assertFalse(any(ORCA.missing_speech(request, intended, completed).values()))

    def test_page_reading_requires_completed_utterances(self):
        request = {"readPage": True, "expectSpeech": ["Project settings"]}
        self.assertEqual(
            ORCA.missing_speech(request, ["Project settings"], [])["missingSpeech"],
            ["Project settings"],
        )
        self.assertFalse(any(ORCA.missing_speech(request, [], ["Project settings"]).values()))


if __name__ == "__main__":
    unittest.main()
