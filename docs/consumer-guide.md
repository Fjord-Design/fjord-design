# Use the Fjord foundation

Run `mise run build` from the repository root. It validates the design and writes three portable formats for Night, Day and Aurora.

| Example output | Use |
| --- | --- |
| `dist/night.dtcg.json` | Typed tokens with aliases and metadata |
| `dist/night.resolved.json` | Flat paths, resolved values, original values and source-layer provenance |
| `dist/night.tokens.css` | Generic CSS custom properties |

The other modes use `day` and `aurora` in their filenames. These exports contain design data. Your implementation supplies components, layout, runtime preferences and target-specific configuration.

The DTCG export is a self-contained document for the selected mode. It preserves value aliases and unknown vendor `$extensions` at the root, group and token levels. Export metadata is merged with the original Fjord metadata, retaining the design name, design seeds and asset manifest references. Resolved JSON and CSS contain resolved values instead; they are not lossless DTCG interchange formats. Fjord still uses its [supported-type and sRGB profile](token-contract.md#format-profile), not every DTCG type or colour space.

## Choose an accent

Mode and accent are independent: `--mode night`, `day` or `aurora` selects the surfaces and complete palette; `--theme` selects an accent layer. Each of these themes works with all three modes:

| Accent | Colour | Theme path |
| --- | --- | --- |
| Fern | Green | `examples/themes/fern.tokens.json` |
| Water | Blue | `examples/themes/water.tokens.json` |
| Mountain | Violet | `examples/themes/mountain.tokens.json` |
| Cabin | Wood brown | `examples/themes/cabin.tokens.json` |
| Hearth | Ember-orange | `examples/themes/hearth.tokens.json` |

For example, export Day with the Water accent:

```sh
mise exec -- python3 -m fjord export --mode day --format resolved \
  --theme examples/themes/water.tokens.json \
  --output dist/day-water.resolved.json
```

Change the mode and theme path to select any combination. Omit `--theme` to retain each mode's unchanged default appearance; Fern is an explicit theme, not a new default. These themes change `accent`, `accent-hover`, `accent-soft`, `accent-ink` and `selection`. `accent-pressed` follows `accent-hover`, `focus-ring` follows `accent`, and `on-accent` remains mode-provided. Status, syntax, ANSI and chart colours keep their mode values.

## Inherit and override

Composition follows **base → mode → theme → overrides**. Omitted values remain inherited. Later layers can replace existing semantic or component token definitions while preserving their types. Aliases continue to follow the selected mode.

```sh
mise exec -- python3 -m fjord export --mode day --format resolved \
  --theme examples/themes/fern.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/day-fern-compact.resolved.json
```

Copy an example theme into your own project when creating a theme. Keep its `$extensions.org.xerrion.fjord` metadata, currently contract `0.5.0`, schema `2` and DTCG `2025.10`, and supply complete token definitions for the paths you change. Only `0.5.x` layers are accepted; `0.4.x` and earlier files must be migrated. Use aliases to semantic roles when a value should adapt to each mode. `--theme` replaces the default `tokens/themes/fjord.tokens.json` layer; repeat `--override` to apply further files in order.

Use `--mode night`, `day` or `aurora`; Night is the CLI default. Each requested composition validates its own contrast, so test every mode your implementation offers. The [token contract](token-contract.md) defines allowed overrides and failure behaviour.

### Write aliases in layers

Prefer readable `{token.path}` aliases for whole-token reuse. A theme or override is an authored composition fragment: its targets may live in base, the selected mode or an earlier layer, rather than in that file. For example, this complete layer reasserts the existing primary-button binding without copying its mode-dependent colour:

```json
{
  "$extensions": {
    "org.xerrion.fjord": {
      "schemaVersion": 2,
      "contractVersion": "0.5.0",
      "dtcgVersion": "2025.10"
    }
  },
  "component": {
    "button": {
      "primary": {
        "background": {
          "$value": "{semantic.color.accent}",
          "$description": "Primary button background."
        }
      }
    }
  }
}
```

The omitted `$type` is derived from the alias target (`color`); it is not guessed from a literal value. A containing group's `$type` can also supply a type. Each replacement still needs its complete `$value` and non-empty `$description`, and its effective type must match the existing role. The fragment above is not a self-contained interchange document because it does not define `semantic.color.accent`; build or export the composed document before exchanging it with another DTCG tool.

The equivalent whole-value pointer is `"$value": {"$ref": "#/semantic/color/accent/$value"}`. For property-level reuse, use a local JSON Pointer rather than extending a brace path into a value's fields. Pointers support nested properties, array indices and metadata, subject to the expected type and the same namespace rules as brace aliases. External files and URLs are not fetched. See the [pointer example and type rules](token-contract.md#references-and-types).

Groups can use [`$root` and `$extends`](token-contract.md#groups-and-root) for explicit base tokens and local group inheritance. Nested groups merge, but a local token definition replaces an inherited token in full. These features do not relax the project contract: themes and overrides may only change existing semantic or component paths, cannot add primitive definitions, and must preserve required roles and types. Use the composed DTCG export, not an individual fragment, when another tool needs all reference targets.

For an alias-preserving DTCG document for one mode, use CLI `build`. It produces the same document as `export --format dtcg`; the default filenames differ. `mise run build` calls `export` for all three modes and all three formats:

```sh
mise exec -- python3 -m fjord build --mode night \
  --theme examples/themes/soft-windows.tokens.json \
  --output dist/night-soft.tokens.json
```

## Bind to design roles

Use semantic roles such as `semantic.color.bg-base`, `semantic.color.ink` and `semantic.color.accent`. Component tokens bind those roles to design intent; their presence does not supply a component implementation.

CSS property names start with `--fjord-token-` and replace token-path dots with hyphens. A reserved `$root` path segment remains explicit and is escaped as `\$root` in CSS; it is not renamed to the ordinary name `root`. For example:

```css
@import "./night.tokens.css";

body {
  background: var(--fjord-token-semantic-color-bg-base);
  color: var(--fjord-token-semantic-color-ink);
}
```

A typography composite expands into font-family, font-size, font-weight, letter-spacing and line-height properties. A shadow may be a single object or an ordered array; an alias used as an array element must resolve to one shadow object, never another array. Whole-token aliases to shadow arrays are valid, but nested arrays are rejected rather than flattened. Dimensions and durations retain their units. Colours become sRGB HEX, including alpha where applicable. Shared contrast checks cover original values and their rounded HEX representation. Resolved JSON preserves numeric precision and effective types, including alias-derived types.

A consumer chooses how to scope styles or switch modes. KDE, Neovim, VS Code and other implementations translate semantic roles into their own target names. Record unsupported features and any substitutions instead of silently changing the shared source.

## Reuse assets

Copy required assets from `assets/`, retaining their manifests and original licences where applicable.

| Assets | Consumer responsibility |
| --- | --- |
| [Fonts](../assets/fonts/README.md) | Load the bundled WOFF2 files or choose documented fallbacks; preserve OFL notices |
| [Icons](../assets/icons/README.md) | Use the supplied SVGs, semantic names, `currentColor` and MIT notice |
| [Brand](../assets/brand/README.md) | Choose the correct background variant and preserve proportions and clear space |

The foundation does not install fonts, generate framework wrappers or package an application theme. Font weights, layout dimensions, icon sizes and other rules are in the tokens and [design guidelines](design-guidelines.md).

## Verify the implementation

Run `mise run validate` when changing design sources and `mise run ci` before contributing changes. A successful export establishes token validity and shared checks. It does not establish rendered contrast, keyboard behaviour, accessibility or support for a particular application version.

Record the Fjord contract version your implementation consumes. Verify your own mappings, asset paths, interactions and native rendering before releasing it. Implementation dependencies and releases remain in the consuming project.

## Migrate to contract 0.5

Contract 0.5 is a breaking cutover from 0.4.x and earlier. It removes all 32 deprecated semantic roles and renames three primitive background paths. The modern roles, current design values, default mode appearances and five accent themes are unchanged. Each mode now supplies 87 semantic colour roles instead of 88.

### Update layer metadata

Source and example layers now declare contract `0.5.0`. Custom themes and overrides must use a compatible `0.5.x` version; `0.4.x` and earlier layers are rejected, even if they already use modern token paths. Update the Fjord metadata block after migrating the paths below:

```json
{
  "$extensions": {
    "org.xerrion.fjord": {
      "schemaVersion": 2,
      "contractVersion": "0.5.0",
      "dtcgVersion": "2025.10"
    }
  }
}
```

Schema `2`, DTCG `2025.10` and **base → mode → theme → overrides** composition are unchanged. Copy current metadata from an example rather than only changing the version of an otherwise incompatible layer.

### Replace removed paths

The following 32 removals are all within the `semantic` namespace. Replace both token definitions and every alias reference to them, including references inside typography and shadow composites:

| Removed semantic paths | Replacement paths | Suffixes / meaning | Count |
| --- | --- | --- | --- |
| `semantic.spacing.space-<number>` | `semantic.spacing.step-<number>` | `1`, `2`, `3`, `4`, `5`, `6`, `8`, `10`, `12`, `16` | 10 |
| `semantic.radius.radius-<name>` | `semantic.radius.<name>` | `none`, `window`, `xs`, `panel`, `control`, `full` | 6 |
| `semantic.typography.type-<name>` | `semantic.typography.<name>` | `display`, `title-1`, `title-2`, `title-3`, `title-4`, `lead`, `body`, `body-strong`, `small`, `label`, `caption`, `overline`, `code`, `code-small`, `code-italic` | 15 |
| `semantic.color.bg-active` | `semantic.color.bg-pressed` or `semantic.color.bg-selected` | Transient press or persistent neutral selection, respectively | 1 |

Use `bg-pressed` while a control is pressed. Use `bg-selected` for an item that remains selected after the interaction. Accent-backed selection continues to use `accent-soft` and `accent-ink`; `selection` remains the text/editor selection role. `accent-pressed` continues to follow `accent-hover` unless overridden independently.

The primitive rename is separate: replace `primitive.color.<mode>.bg-active` with `primitive.color.<mode>.bg-pressed` for each of `night`, `day` and `aurora`. The original primitive values are retained. Spacing and radius primitive names are unchanged; do not apply the semantic prefix removals to them.

Move theme and override assignments to the modern semantic paths, not just alias targets. The supplied compact-spacing example uses `semantic.spacing.step-4`, and soft-windows uses `semantic.radius.window`. Removed paths are unknown roles, not compatibility aliases; references to them cannot resolve.

### Update consumers and regenerate exports

The removed aliases and their CSS custom properties, DTCG paths and resolved JSON keys are no longer exported. Update external bindings and mappings as well as token layers. CSS names still replace token-path dots with hyphens, so, for example, `--fjord-token-semantic-spacing-space-4` becomes `--fjord-token-semantic-spacing-step-4`. JSON lookups for `semantic.spacing.space-4` become `semantic.spacing.step-4`. Apply the same naming rule to the other removed paths and to the primitive background rename.

For consumers predating the Python module rename, replace `python3 -m fjord_tokens` with `python3 -m fjord` and imports from `fjord_tokens` with `fjord`, retaining the relevant submodule names. The old module is not a compatibility entry point.

Regenerate exports for every mode and theme your implementation offers, then verify its bindings, pressed and selected states and rendered output. The CLI cannot inspect external consumers for removed CSS properties or JSON paths.

### Deprecation support remains available

No definitions in Fjord's shipped sources, default theme or example layers are currently deprecated. Generic DTCG `$deprecated` metadata validation and use-based CLI diagnostics remain available for future deprecations. Aliases to deprecated roles, including references inside composites, and explicit theme or override assignments to deprecated paths produce warnings. Warnings identify the source, deprecated role and replacement; deprecated definitions alone do not produce warnings. This facility does not restore any removed 0.4 alias.

