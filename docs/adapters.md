# Platform adapters

Run `mise run adapters:wezterm` to build the reference adapter in `dist/adapters/wezterm/`. It writes `night.toml`, `day.toml`, `aurora.toml` and `report.json`. The three schemes are byte-identical to the existing [WezTerm token export](token-export.md); the report adds explicit mapping coverage and fallback information. Generation does not change application settings or install themes.

Run `mise run adapters:zed` to generate the first editor adapter. The [Zed guide](../adapters/zed/README.md) covers installation, retained-text highlight fallbacks and version limits; [XER-140 verification](verification/xer-140.md) records the checks.

Run `mise run adapters:vscode` to generate the [VS Code themes](../adapters/vscode/README.md) and `dist/release/fjord-themes-0.1.0.vsix`. The package contains only declarative themes, its manifest and documentation/licence. It is built locally without contacting Marketplace. [XER-134 verification](verification/xer-134.md) records the tested version and remaining limits.

Adapters consume semantic tokens through the platform-independent Python core. Target-specific manifests, serializers, fixtures and installation instructions belong in `adapters/<platform>/`. A generated file establishes format and mapping evidence; native support requires separate application tests. See [ADR 0001](decisions/0001-platform-boundaries.md).

## Declare a mapping

Start from [`adapters/wezterm/adapter.json`](../adapters/wezterm/adapter.json). Schema version `1` requires these fields:

| Field | Contract |
| --- | --- |
| `schemaVersion`, `id` | Integer `1` and a lower-case hyphenated adapter identifier. |
| `target` | Non-empty `application`, `version` and `os` strings identifying the test target. These fields do not claim support. |
| `modes` | A non-empty list containing each supported Fjord mode at most once. |
| `mapping` | Flat target names mapped to existing `semantic.*` token paths. Dots in a target name are literal; the serializer owns nesting. |
| `requiredRoles` | Unique source paths that must appear in `mapping`; an unsupported-role explanation cannot satisfy a required mapping. |
| `unsupported` | Existing semantic source paths mapped to specific explanations. A role cannot be both mapped and unsupported. |
| `fallbacks` | Non-empty explanations for `colorSpace`, `fonts`, `radius`, `shadows`, `motion` and `interaction`. |

The loader rejects unknown manifest fields, invalid modes, missing required mappings, duplicate JSON keys, unknown token paths and mapped/unsupported conflicts. The mapper validates the source project, composes each requested mode and checks source and HEX contrast before returning output. Theme layers and ordered overrides use the same rules as other token exports.

```python
from fjord_tokens.adapters import map_adapter

modes, report = map_adapter(
    root=".",
    manifest_path="adapters/wezterm/adapter.json",
    theme="examples/themes/fern.tokens.json",
)
night_cursor = modes["night"]["cursor_bg"]
```

Run this from the repository root with `mise exec -- python3` so the configured `PYTHONPATH=src` applies. Colours become sRGB HEX using the core's existing conversion; other semantic values retain their resolved DTCG representation. No platform runtime is imported. A serializer must reject unsupported target names and value types before writing its format.

## Inspect coverage and fallbacks

Each mode in `report.json` includes `mapped`, `unsupported` and `unmapped`. Mapped entries record target name, source role, type, emitted value, original alias/value and source layer. Unsupported entries retain the manifest's explanation. Unmapped entries list every remaining `semantic.color.*` role, including roles that belong to web components rather than the target application. They are visible coverage gaps; they are not silently counted as supported. Non-colour roles are included when explicitly mapped or declared unsupported; their remaining application limits belong in `fallbacks`.

The report also records the adapter, test target, contract version and fallbacks. Stable key ordering and source-layer labels keep it deterministic without embedding absolute checkout paths. Rendered contrast on extra target-specific surfaces still needs the adapter's own checks; core contrast validation covers the shared documented pairs.

The reference colour scheme leaves font installation, ligatures, window radius, shadows, animation, key bindings and shell behaviour under the user's control. The manifest explains these limits. It maps cursor and selection colours but cannot guarantee the appearance of programs that choose their own colours.

## Generate a variant

```sh
mise exec -- python3 adapters/wezterm/generate.py \
  --theme examples/themes/fern.tokens.json \
  --output-dir dist/adapters/wezterm-fern
```

Repeat `--override path` to apply additional layers in order. `--root` selects another Fjord source root. The command returns `0` after generation, `1` for invalid sources, mappings or filesystem failures, and `2` for invalid command arguments. Errors go to stderr. All modes are generated and checked before writing; the writer checks every destination before the first write and refuses project source directories, existing root files, protected inputs and paths escaping the destination. Individual file replacements are atomic; a filesystem failure can leave a partially updated bundle. Regenerate into a new directory when retaining an existing bundle matters.

## Install, upgrade and restore WezTerm

Use the [WezTerm installation example](token-export.md#wezterm) after preserving your existing configuration. Copy one generated TOML to an unused filename and explicitly point the colour configuration at it. The adapter does not modify `wezterm.lua` or choose fonts.

For an upgrade, generate into a new directory, inspect the coverage report, compare the scheme and test the new file before changing the configured filename. Retain the previous scheme and configuration for restoration. To uninstall, restore the previous colour setting or saved configuration, then remove only the Fjord files you added. The manifest's macOS/WezTerm version remains a test target until native results are recorded; regenerated equality with XER-116 is not a new native verification.

Fjord's adapter code and manifests use the repository's [MIT licence](../LICENSE). Retain any target schema or other upstream asset licences with the relevant adapter.
