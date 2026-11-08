"""Generate a bounded KDE/Qt colour proof of concept without applying system settings."""
import argparse
import json
from pathlib import Path
import re
import sys

from fjord_tokens.adapters import map_adapter, write_adapter_files
from fjord_tokens.dtcg import TokenError, color_from_hex
from fjord_tokens.project import contrast_value
ROOT=Path(__file__).resolve().parents[2]


def bundle(root=ROOT,theme=None,overrides=()):
    modes,report=map_adapter(root,'adapters/kde/adapter.json',theme,overrides)
    files={}
    expected = {f"Colors:{section}.{field}" for section in ("View","Window","Button","Tooltip","Complementary","Header","Selection") for field in ("BackgroundNormal","BackgroundAlternate","ForegroundNormal","ForegroundInactive","ForegroundLink","ForegroundVisited","ForegroundActive","ForegroundNegative","ForegroundNeutral","ForegroundPositive","DecorationFocus","DecorationHover")}
    for mode,values in modes.items():
        if set(values) != expected: raise TokenError(f"KDE: missing targets {sorted(expected-values.keys())}; unknown targets {sorted(values.keys()-expected)}")
        sections={}
        for target,value in sorted(values.items()):
            if not re.fullmatch(r'Colors:(View|Window|Button|Tooltip|Complementary|Header|Selection)\.[A-Za-z]+',target) or not isinstance(value,str) or not re.fullmatch(r'#[0-9a-f]{6}',value):
                raise TokenError(f'KDE: invalid mapped target/value {target}')
            section,key=target.split('.')
            sections.setdefault(section,{})[key]=','.join(str(int(value[i:i+2],16)) for i in (1,3,5))
        pairs=[]
        for section in sections:
            background=values[section+'.BackgroundNormal']
            for key in ['ForegroundNormal','ForegroundInactive','ForegroundLink','ForegroundVisited','ForegroundNegative','ForegroundNeutral','ForegroundPositive']:
                foreground=values[section+'.'+key]
                ratio=contrast_value(color_from_hex(foreground),color_from_hex(background))
                if ratio<4.5:raise TokenError(f'KDE {mode} {section}.{key}: contrast {ratio:.6f} below 4.5')
                pairs.append({'section':section,'foreground':foreground,'background':background,'ratio':ratio,'minimum':4.5})
        report['modes'][mode]['contrast']=pairs
        lines=[f'# Fjord KDE proof of concept; MIT; mode {mode}.','[General]',f'Name=Fjord {mode.title()} (POC)',f'ColorScheme=Fjord{mode.title()}','']
        for section,entries in sorted(sections.items()):lines+=['['+section+']',*[key+'='+value for key,value in sorted(entries.items())],'']
        files[f'Fjord{mode.title()}.colors']='\n'.join(lines)
    files['report.json']=json.dumps(report,indent=2,allow_nan=False)+'\n'
    for name in ('README.md','LICENSE'):
        files[name]=(Path(root)/'adapters/kde'/name).read_text()
    return files


def main():
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('--root',type=Path,default=ROOT);parser.add_argument('--theme',type=Path);parser.add_argument('--override',action='append',type=Path,default=[]);parser.add_argument('--output-dir',type=Path)
    args=parser.parse_args()
    try:
        write_adapter_files(args.root,args.output_dir or args.root/'dist/adapters/kde',bundle(args.root,args.theme,args.override),[*args.override]+([args.theme]if args.theme else []))
        print('Built isolated KDE colour proof of concept')
    except(TokenError,OSError,UnicodeError)as error:print(f'ERROR {error}',file=sys.stderr);return 1
    return 0

if __name__=='__main__':raise SystemExit(main())
