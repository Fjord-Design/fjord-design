"""Check the existing native WezTerm Lua sandbox without starting a GUI or installing tools."""
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    executable = shutil.which("wezterm")
    if not executable:
        raise SystemExit("NOT RUN: no existing WezTerm runtime; nothing installed")
    with tempfile.TemporaryDirectory(prefix="fjord-wezterm-lua-") as tmp:
        for identity in ("fjord", "work", "gaming"):
            bundle = ROOT / "dist/adapters/wezterm" if identity == "fjord" else ROOT / f"dist/variants/{identity}/wezterm"
            for mode in ("night", "day", "aurora"):
                marker = Path(tmp) / f"{identity}-{mode}.ok"
                config = Path(tmp) / "wezterm.lua"
                config.write_text("local w=require 'wezterm'\nlocal c=w.config_builder()\n"
                    "c.font_size=17\nc.colors={tab_bar={background='#123456'}}\n"
                    f"c=dofile({json.dumps(str(bundle / 'init.lua'))}).apply(c,{json.dumps(mode)},{json.dumps(str(bundle))})\n"
                    "assert(c.font_size==17)\nassert(c.colors.tab_bar.background=='#123456')\n"
                    "assert(c.colors.foreground~=c.colors.background)\nassert(#c.colors.ansi==8 and #c.colors.brights==8)\n"
                    f"local f=assert(io.open({json.dumps(str(marker))},'w'));f:write('PASS');f:close()\nreturn c\n")
                result = subprocess.run([executable, "--config-file", str(config), "show-keys", "--lua"], capture_output=True, text=True, timeout=20)
                # WezTerm can return success after rejecting a config and falling back.
                if result.returncode or "Configuration Error" in result.stderr or not marker.is_file() or marker.read_text() != "PASS":
                    raise RuntimeError(f"WezTerm did not execute {identity}/{mode} assertions: {result.stderr}")
    print("PASS nine native WezTerm Lua configurations: palette and unrelated settings preserved")


if __name__ == "__main__":
    main()
