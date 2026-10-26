"""Check reproducibility of token and specimen outputs, and local document links."""

import re
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def run(*arguments):
    subprocess.run([sys.executable, "-m", "fjord_tokens", *map(str, arguments)], cwd=ROOT, check=True,
                   stdout=subprocess.DEVNULL)


def main():
    semantic_paths = ["semantics.css", "semantics.html", "semantics.json", "theme-preference.js"]
    first_semantics = {name: (ROOT / "dist" / name).read_bytes() for name in semantic_paths}
    run("semantics")
    if first_semantics != {name: (ROOT / "dist" / name).read_bytes() for name in semantic_paths}:
        raise ValueError("semantic role/state output differs between builds")
    if (ROOT / "dist/semantics.css").read_bytes() != (ROOT / "dist/svelte/semantics.css").read_bytes():
        raise ValueError("packaged semantic CSS differs from validated source")
    foundation_paths = ["foundations.css", "foundations.html", "foundations.json", "icons/manifest.json", "icons/LICENSE"]
    first_foundations = {name: (ROOT / "dist" / name).read_bytes() for name in foundation_paths}
    run("foundations")
    if first_foundations != {name: (ROOT / "dist" / name).read_bytes() for name in foundation_paths}:
        raise ValueError("foundation output differs between builds")
    if (ROOT / "dist/foundations.css").read_bytes() != (ROOT / "dist/svelte/foundations.css").read_bytes():
        raise ValueError("packaged foundation CSS differs from the token output")
    def typography_snapshot():
        paths = [ROOT / "dist/typography.html", ROOT / "dist/typography.json", *sorted((ROOT / "dist/fonts").rglob("*"))]
        return {str(path.relative_to(ROOT / "dist")): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in paths if path.is_file()}

    first = typography_snapshot()
    run("typography")
    if first != typography_snapshot():
        raise ValueError("dist typography/font outputs differ between builds")
    for source in (ROOT / "dist/fonts").rglob("*"):
        if source.is_file() and source.read_bytes() != (ROOT / "dist/svelte/fonts" / source.relative_to(ROOT / "dist/fonts")).read_bytes():
            raise ValueError(f"{source}: font package differs from the specimen")
    with tempfile.TemporaryDirectory(prefix="fjord-reproducibility-") as temporary:
        directory = Path(temporary)
        outputs = [(f"{mode}.tokens.json", ["build", "--mode", mode]) for mode in ("night", "day", "aurora")]
        outputs.append(("code-and-terminal.html", ["preview"]))
        outputs.extend((f"palette.{kind}", ["palette", "--format", kind]) for kind in ("html", "json"))
        for name, command in outputs:
            first, second = directory / f"first-{name}", directory / f"second-{name}"
            run(*command, "--output", first)
            run(*command, "--output", second)
            if first.read_bytes() != second.read_bytes() or first.read_bytes() != (ROOT / "dist" / name).read_bytes():
                raise ValueError(f"dist/{name}: output differs between builds")
    for name in ("svelte/index.js", "svelte/index.d.ts", "svelte/ThemeScope.svelte.d.ts", "svelte/LICENSE",
                 "reference/index.html", "storybook/index.html", "storybook/index.json",
                 "typography.html", "typography.json", "fonts/typography.css", "svelte/fonts/typography.css"):
        if not (ROOT / "dist" / name).is_file():
            raise ValueError(f"dist/{name}: required build output is missing")
    count = 0
    for source in [ROOT / "README.md", ROOT / "CONTRIBUTING.md", *sorted((ROOT / "docs").rglob("*.md")),
                   ROOT / "assets/brand/README.md", ROOT / "assets/fonts/README.md", ROOT / "assets/icons/README.md", ROOT / "reference/svelte/README.md", ROOT / "patches/README.md"]:
        for target in re.findall(r"\]\(<?([^\s)>]+)>?\)", source.read_text(encoding="utf-8")):
            if "://" in target or target.startswith(("#", "mailto:")):
                continue
            path = target.split("#", 1)[0]
            if not (source.parent / path).exists():
                raise ValueError(f"{source.relative_to(ROOT)}: missing link target {target}")
            count += 1
    print(f"PASS deterministic token/specimen outputs, reference/catalogue artifacts and {count} local links")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        sys.exit(1)
