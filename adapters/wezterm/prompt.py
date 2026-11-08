"""Optional standalone prompt example; reads the generated palette and never edits shell startup files."""
import argparse
import json
from pathlib import Path
import re
import subprocess


def clean(value):
    return re.sub(r'[\x00-\x1f\x7f-\x9f]', '', str(value))


def render(palette,mode,cwd,environment,duration_ms,plain=False):
    colours=palette[mode]
    def text(role,value):
        value=clean(value)
        if plain:return value
        colour=colours[role]
        return '\033[38;2;'+ ';'.join(str(int(colour[i:i+2],16)) for i in (1,3,5))+'m'+value+'\033[0m'
    try:
        branch=subprocess.run(['git','branch','--show-current'],cwd=cwd,capture_output=True,text=True,timeout=3)
        status=subprocess.run(['git','status','--porcelain'],cwd=cwd,capture_output=True,text=True,timeout=3)
    except (OSError, subprocess.TimeoutExpired):
        branch=subprocess.CompletedProcess([],1,'')
        status=subprocess.CompletedProcess([],1,'')
    context=[text('prompt-path',str(cwd)),text('prompt-branch','branch '+(branch.stdout.strip() or '(detached/no repository)')),text('prompt-environment','env '+environment)]
    if status.returncode==0:
        lines=status.stdout.splitlines();counts={'A':0,'M':0,'D':0,'R':0,'?':0,'U':0}
        for line in lines:
            code=line[:2]
            key='?' if code=='??' else 'U' if 'U'in code or code in ('AA','DD') else next((s for s in ('R','D','A','M')if s in code),None)
            if key:counts[key]+=1
        context.append(text('prompt-branch','git '+(' '.join(f'{key}{value}'for key,value in counts.items()if value)or 'clean')))
    context.extend([text('prompt-duration',f'{duration_ms}ms'),text('prompt-symbol','>')])
    return ' | '.join(context)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode',choices=['night','day','aurora'],default='night');parser.add_argument('--cwd',type=Path,default=Path.cwd());parser.add_argument('--environment',default='local');parser.add_argument('--duration-ms',type=int,default=0);parser.add_argument('--plain',action='store_true')
    args=parser.parse_args()
    if args.duration_ms<0:parser.error('duration-ms must be non-negative')
    palette=json.loads(Path(__file__).with_name('prompt-colors.json').read_text())
    print(render(palette,args.mode,args.cwd,args.environment,args.duration_ms,args.plain))

if __name__=='__main__':main()
