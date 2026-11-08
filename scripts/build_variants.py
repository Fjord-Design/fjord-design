"""Build separately named work/gaming identities and adapter packages from shared overrides."""
import importlib.util
import json
from pathlib import Path

from fjord_tokens.adapters import write_adapter_files
from fjord_tokens.project import Project,check_contrast
from fjord_tokens.theme_css import theme_css

ROOT=Path(__file__).resolve().parents[1]
IDENTITIES=('work','gaming')


def module(root,relative):
    spec=importlib.util.spec_from_file_location('fjord_'+relative.replace('/','_').replace('.','_'),Path(root)/relative)
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result


def json_text(value):return json.dumps(value,ensure_ascii=False,indent=2,allow_nan=False)+'\n'


def bundle(root=ROOT):
    root=Path(root);files={};report={'identities':{},'nativeStatus':'Native support is limited to the recorded application/version/identity checks; generation is not native acceptance.'}
    package=module(root,'scripts/package_vscode.py')
    for identity in IDENTITIES:
        theme=root/f'examples/themes/{identity}.tokens.json';title='Fjord '+identity.title();details={}
        for mode in ('night','day','aurora'):
            composition=Project(root).compose(mode,theme);check_contrast(composition)
            base=Project(root).compose(mode)
            if composition.tokens.keys()!=base.tokens.keys():raise ValueError('Variants must preserve token names')
            for path,token in base.tokens.items():
                if path.startswith('component.') and token!=composition.tokens[path]:raise ValueError('Variants must preserve component bindings')
        files[f'{identity}.css']=theme_css(root,identity,theme)
        for platform in ('wezterm','zed','vscode','neovim','kde'):
            generator=module(root,f'adapters/{platform}/generate.py')
            output=generator.build_bundle(root,theme) if platform=='vscode' else generator.bundle(root,theme,name='fjord-'+identity) if platform=='neovim' else generator.bundle(root,theme)
            if platform=='zed':
                family=json.loads(output['themes/fjord.json']);family['name']=title
                for entry in family['themes']:entry['name']=entry['name'].replace('Fjord',title,1)
                output['themes/fjord.json']=json_text(family)
                manifest=output['extension.toml'].decode() if isinstance(output['extension.toml'],bytes) else output['extension.toml']
                output['extension.toml']=manifest.replace('id = "fjord"',f'id = "fjord-{identity}"').replace('name = "Fjord"',f'name = "{title}"')
            elif platform=='vscode':
                manifest=json.loads(output['package.json']);manifest['name']='fjord-themes-'+identity;manifest['displayName']=title
                for entry in manifest['contributes']['themes']:
                    entry['label']=entry['label'].replace('Fjord',title,1)
                    if 'id' in entry:entry['id']=entry['id'].replace('Fjord',title,1)
                for path,content in list(output.items()):
                    if path.startswith('themes/') and path.endswith('.json'):
                        document=json.loads(content);document['name']=document['name'].replace('Fjord',title,1);output[path]=json_text(document)
                output['package.json']=json_text(manifest)
                output['README.md']=f'# {title} for VS Code\n\nInstall this separately named local variant with Install from VSIX. Select a {title} mode. Uninstall this variant to restore the original Fjord extension; record the previous theme preference first. No settings file is replaced. Native support is not inferred from generation.\n'
                files[f'{identity}/fjord-themes-{identity}-0.1.0.vsix']=package.package_files(output)
            elif platform=='wezterm':
                for path,content in list(output.items()):
                    if path.endswith('.toml'):output[path]=content.replace('name = "Fjord ',f'name = "{title} ')
            elif platform=='kde':
                for path,content in list(output.items()):
                    if path.endswith('.colors'):output[path]=content.replace('Name=Fjord ',f'Name={title} ').replace('ColorScheme=Fjord',f'ColorScheme=Fjord{identity.title()}')
            details[platform]={'files':sorted(output),'report':json.loads(output['report.json'])}
            for path,content in output.items():files[f'{identity}/{platform}/{path}']=content
        report['identities'][identity]=details
    files['report.json']=json_text(report)
    return files


def main():
    files=bundle();write_adapter_files(ROOT,ROOT/'dist/variants',files,[ROOT/f'examples/themes/{name}.tokens.json'for name in IDENTITIES]);print('Built work/gaming identities, five adapter targets and local VSIX variants')

if __name__=='__main__':main()
