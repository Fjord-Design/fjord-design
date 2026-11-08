import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("fjord_prompt", ROOT / "adapters/wezterm/prompt.py")
PROMPT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROMPT)
PALETTE = {"night": {role: "#ABCDEF" for role in ("prompt-path", "prompt-branch", "prompt-environment", "prompt-duration", "prompt-symbol")}}


class WezTermDailyTests(unittest.TestCase):
    def test_prompt_handles_no_git_and_timeout_without_losing_context(self):
        for error in (FileNotFoundError("git"), subprocess.TimeoutExpired("git", 3)):
            with self.subTest(error=error), patch.object(PROMPT.subprocess, "run", side_effect=error):
                value = PROMPT.render(PALETTE, "night", Path("/project"), "local", 42, True)
                self.assertIn("/project", value)
                self.assertIn("42ms", value)
                self.assertNotIn("\x1b", value)

    def test_git_states_are_textual_and_untrusted_controls_are_removed(self):
        responses = [subprocess.CompletedProcess([], 0, "feature\x1b[31m\n"), subprocess.CompletedProcess([], 0, "?? new\n M changed\nUU conflict\n")]
        with patch.object(PROMPT.subprocess, "run", side_effect=responses):
            value = PROMPT.render(PALETTE, "night", Path("/project"), "dev\x07\n", 0, True)
        for text in ("?1", "M1", "U1", "env dev", "branch feature"):
            self.assertIn(text, value)
        self.assertNotIn("\x1b", value)
        self.assertNotIn("\x07", value)
