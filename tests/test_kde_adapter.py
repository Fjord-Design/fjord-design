import importlib.util
import json
import configparser
from pathlib import Path
import unittest
from unittest.mock import patch
from fjord_tokens.dtcg import TokenError
ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('fjord_kde',ROOT/'adapters/kde/generate.py');KDE=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(KDE)

class KdeTests(unittest.TestCase):
    def test_deterministic_rgb_source_mapping_and_contrast(self):
        for identity in (None,ROOT/'examples/themes/fern.tokens.json',ROOT/'examples/themes/work.tokens.json',ROOT/'examples/themes/gaming.tokens.json'):
            files=KDE.bundle(ROOT,identity);self.assertEqual(files,KDE.bundle(ROOT,identity));report=json.loads(files['report.json'])
            for mode,entry in report['modes'].items():
                self.assertFalse(entry['unmapped']);self.assertTrue(all(pair['ratio']>=4.5 for pair in entry['contrast']))
                self.assertIn('[Colors:Selection]',files[f'Fjord{mode.title()}.colors'])
                parsed=configparser.ConfigParser();parsed.optionxform=str;parsed.read_string(files[f'Fjord{mode.title()}.colors'])
                for target,mapped in entry['mapped'].items():
                    section,field=target.split('.');colour=mapped['value']
                    self.assertEqual(parsed[section][field],','.join(str(int(colour[i:i+2],16))for i in (1,3,5)))
    def test_missing_and_unknown_target_fail_before_output(self):
        modes,report=KDE.map_adapter(ROOT,'adapters/kde/adapter.json')
        del modes['night']['Colors:View.ForegroundNormal']
        with patch.object(KDE,'map_adapter',return_value=(modes,report)),self.assertRaisesRegex(TokenError,'missing targets'):KDE.bundle(ROOT)

if __name__=='__main__':unittest.main()
