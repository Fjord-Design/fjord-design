"""Refresh the committed icon data from the validated local SVG subset."""

from pathlib import Path

from fjord_tokens.icons import icon_module, load_icons


root = Path(__file__).resolve().parents[1]
_, bodies = load_icons(root)
(root / "reference/svelte/src/lib/icons.generated.ts").write_text(icon_module(bodies), encoding="utf-8")
