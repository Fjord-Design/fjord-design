"""Authorized isolated Linux/XWayland WezTerm fixture capture; does not edit user configuration."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import tomllib

ROOT=Path(__file__).resolve().parents[2]


def run(*args,**kwargs):return subprocess.run(list(map(str,args)),check=True,capture_output=True,text=True,timeout=15,**kwargs).stdout

def wait_for(check,seconds=12):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        result=check()
        if result:return result
        time.sleep(.1)
    raise RuntimeError('Timed out waiting for the owned native test window')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output-dir',type=Path,default=ROOT/'dist/verification/wezterm-native');parser.add_argument('--identity',choices=['fjord','work','gaming'],default='fjord');args=parser.parse_args()
    for tool in ('wezterm','xdotool','import','magick'):
        if not shutil.which(tool):raise SystemExit(f'NOT RUN: missing existing native tool {tool}; nothing installed')
    out=args.output_dir.resolve();out.mkdir(parents=True,exist_ok=True)
    bundle=ROOT/'dist/adapters/wezterm' if args.identity=='fjord'else ROOT/f'dist/variants/{args.identity}/wezterm'
    evidence={'identity':args.identity,'version':run('wezterm','--version').strip(),'backend':'XWayland in a KDE Wayland session; isolated config','modes':{}}
    for mode in ('night','day','aurora'):
        with tempfile.TemporaryDirectory(prefix='fjord-wezterm-native-')as tmp:
            directory=Path(tmp);selection=directory/'selection.txt';request=directory/'select.txt';config=directory/'wezterm.lua';window_class='FjordM4'+mode+str(os.getpid())
            config.write_text("local wezterm=require 'wezterm'\nlocal c=wezterm.config_builder()\n"+
                f"c=dofile({json.dumps(str(bundle/'init.lua'))}).apply(c,{json.dumps(mode)},{json.dumps(str(bundle))})\n"+
                "c.enable_wayland=false\nc.font_size=13\nc.initial_cols=110\nc.initial_rows=28\nc.enable_tab_bar=false\nc.window_close_confirmation='NeverPrompt'\nc.default_cursor_style='SteadyBlock'\nc.cursor_blink_rate=0\nc.check_for_updates=false\n"+
                "local selected=false\n"+
                f"wezterm.on('update-status',function(window,pane) local requested=io.open({json.dumps(str(request))},'r');if requested then requested:close();if not selected then selected=true;window:perform_action(wezterm.action.ActivateCopyMode,pane);window:perform_action(wezterm.action.CopyMode 'MoveToScrollbackTop',pane);window:perform_action(wezterm.action.CopyMode 'MoveToStartOfLine',pane);window:perform_action(wezterm.action.CopyMode{{SetSelectionMode='Cell'}},pane);for i=1,25 do window:perform_action(wezterm.action.CopyMode 'MoveRight',pane) end end end;local f=io.open({json.dumps(str(selection))},'w');if f then f:write(window:get_selection_text_for_pane(pane));f:close() end end)\nreturn c\n")
            log=(out/f'{args.identity}-{mode}.log').open('w')
            process=subprocess.Popen(['wezterm','--config-file',str(config),'start','--always-new-process','--class',window_class,'--cwd',str(ROOT),'sh',str(ROOT/'adapters/wezterm/fixtures.sh')],stdout=log,stderr=log,start_new_session=True)
            window=None
            try:
                def find():
                    result=subprocess.run(['xdotool','search','--onlyvisible','--class',window_class],capture_output=True,text=True,timeout=2)
                    return result.stdout.strip().splitlines()[0]if result.returncode==0 and result.stdout.strip()else None
                window=wait_for(find)
                run('xdotool','windowactivate','--sync',window)
                wait_for(lambda:run('xdotool','getactivewindow').strip()==window)
                # A native key observation after startup is needed before capturing the stable fixture.
                time.sleep(.6)
                name=out/f'{args.identity}-{mode}'
                run('import','-window',window,str(name)+'-base.png')
                request.write_text('select fixture')
                selected=wait_for(lambda:selection.read_text()if selection.exists()and selection.read_text().strip()else None)
                run('import','-window',window,str(name)+'-selection.png')
                scheme=tomllib.loads((bundle/f'{mode}.toml').read_text())['colors']
                histogram=run('magick',str(name)+'-base.png','-format','%c','histogram:info:')
                expected=set(scheme['ansi']+scheme['brights']+[scheme['background']])
                missing=[colour for colour in sorted(expected)if colour.lower()not in histogram.lower()]
                evidence['modes'][mode]={'selectedText':selected,'paletteColours':len(expected),'missingPalettePixels':missing,'base':str(name.name)+'-base.png','selection':str(name.name)+'-selection.png','truecolourExpected':'#8cb4c8','truecolourPixelsPresent':'#8cb4c8'in histogram.lower()}
                if missing:raise RuntimeError(f'{mode}: expected palette pixels missing: {missing}')
            finally:
                if window:
                    subprocess.run(['xdotool','windowclose',window],capture_output=True,timeout=5)
                try:process.wait(timeout=5)
                except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGTERM);process.wait(timeout=5)
                log.close()
    (out/f'{args.identity}-results.json').write_text(json.dumps(evidence,indent=2)+'\n')
    print(f'PASS isolated native WezTerm fixture, palette pixels and selection: {args.identity}')

if __name__=='__main__':main()
