import importlib.util
import json
from pathlib import Path
import unittest

from fjord_tokens.dtcg import TokenError

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('fjord_neovim', ROOT / 'adapters/neovim/generate.py')
ADAPTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ADAPTER)


class NeovimTests(unittest.TestCase):
    def test_determinism_coverage_highlights_and_both_identities(self):
        for identity in (None, ROOT / 'examples/themes/fern.tokens.json'):
            bundle = ADAPTER.bundle(ROOT, identity)
            self.assertEqual(bundle, ADAPTER.bundle(ROOT, identity))
            report = json.loads(bundle['report.json'])
            for mode, details in report['modes'].items():
                self.assertEqual(details['unmapped'], [])
                self.assertTrue(details['contrast'])
                self.assertTrue(all(pair['ratio'] >= pair['minimum'] for pair in details['contrast']))
                code = bundle[f'colors/fjord-{mode}.lua']
                self.assertIn('vim.api.nvim_set_hl', code)
                self.assertNotIn('NONE', code)
                self.assertEqual(details['highlights']['Visual']['fg'], details['highlights']['Normal']['fg'])
                self.assertTrue(details['highlights']['Comment']['italic'])
                self.assertTrue(details['highlights']['DiagnosticUnderlineWarn']['underdashed'])
            self.assertIn('LICENSE', bundle)

    def test_wrong_colour_invalid_target_missing_palette_and_code_injection_fail(self):
        manifest = json.loads((ROOT / 'adapters/neovim/adapter.json').read_text())
        from fjord_tokens.adapters import map_adapter
        modes, _ = map_adapter(ROOT, 'adapters/neovim/adapter.json')
        for target, value in [('Normal.fg', 'NONE'), ('bad;code.fg', '#ffffff'), ('terminal_color_16', '#ffffff')]:
            values = dict(modes['night'], **{target: value})
            with self.subTest(target=target), self.assertRaises(TokenError):
                ADAPTER.highlights(values, dict(manifest['mapping'], **{target: 'semantic.color.ink'}))
        values = dict(modes['night'])
        del values['terminal_color_0']
        with self.assertRaises(TokenError): ADAPTER.highlights(values, manifest['mapping'])
        with self.assertRaises(TokenError): ADAPTER.bundle(ROOT, name='fjord";evil')

    def test_contrast_regression_stops_bundle(self):
        groups = {'Normal': {'fg': '#000000', 'bg': '#000000'}}
        with self.assertRaisesRegex(TokenError, 'contrast'):
            ADAPTER.contrast(groups, ['#ffffff'] * 16, 'night')


if __name__ == '__main__': unittest.main()
