import importlib.util
import json
from pathlib import Path
import unittest
from fjord_tokens.project import Project
ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('fjord_variants',ROOT/'scripts/build_variants.py')
VARIANTS=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(VARIANTS)

class VariantTests(unittest.TestCase):
    def test_all_targets_names_and_tokens_are_preserved(self):
        files=VARIANTS.bundle(ROOT)
        self.assertEqual(files,VARIANTS.bundle(ROOT))
        for name in VARIANTS.IDENTITIES:
            self.assertIn(f'[data-fjord-identity="{name}"]',files[f'{name}.css'])
            report=json.loads(files['report.json'])['identities'][name]
            self.assertEqual(set(report),{'wezterm','zed','vscode','neovim','kde'})
            family=json.loads(files[f'{name}/zed/themes/fjord.json'])
            self.assertEqual(family['name'],'Fjord '+name.title())
            manifest=json.loads(files[f'{name}/vscode/package.json'])
            self.assertEqual(manifest['name'],'fjord-themes-'+name)
            self.assertIn(f'{name}/fjord-themes-{name}-0.1.0.vsix',files)
            self.assertIn(f'{name}/neovim/colors/fjord-{name}-day.lua',files)
            for mode in ('night','day','aurora'):
                base=Project(ROOT).compose(mode);variant=Project(ROOT).compose(mode,ROOT/f'examples/themes/{name}.tokens.json')
                self.assertEqual(base.tokens.keys(),variant.tokens.keys())
                for path,token in base.tokens.items():
                    if path.startswith('component.'):self.assertEqual(token,variant.tokens[path])

if __name__=='__main__':unittest.main()
