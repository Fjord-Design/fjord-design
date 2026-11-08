"""Run the optional native Neovim check without installing a runtime or touching user settings."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    executable = os.environ.get("FJORD_NVIM") or shutil.which("nvim")
    if not executable:
        raise SystemExit("NOT RUN: Neovim is unavailable. Install it only with owner permission, or set FJORD_NVIM to an existing binary.")
    with tempfile.TemporaryDirectory(prefix="fjord-neovim-") as directory:
        env = dict(os.environ, FJORD_NVIM_BUNDLE=str(ROOT / "dist/adapters/neovim"))
        for key in ("XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME", "XDG_CACHE_HOME"):
            env[key] = str(Path(directory) / key.lower())
        result = subprocess.run([executable, "--headless", "--clean", "-i", "NONE",
                                 "--cmd", "lua vim.opt.runtimepath:prepend(vim.env.FJORD_NVIM_BUNDLE)",
                                 "-l", str(ROOT / "adapters/neovim/fixtures/check.lua")], env=env, timeout=60)
        raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
