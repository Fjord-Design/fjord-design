# Use the Fjord foundation

Run `mise run build` from the repository root. It validates the design and writes three portable formats for Night, Day and Aurora.

| Example output | Use |
| --- | --- |
| `dist/night.dtcg.json` | Typed tokens with aliases and metadata |
| `dist/night.resolved.json` | Flat paths, resolved values, original values and source-layer provenance |
| `dist/night.tokens.css` | Generic CSS custom properties |

The other modes use `day` and `aurora` in their filenames. These exports contain design data. Your implementation supplies components, layout, runtime preferences and target-specific configuration.

## Inherit and override

Composition follows **base → mode → theme → overrides**. Omitted values remain inherited. Later layers can replace existing semantic or component token definitions while preserving their types. Aliases continue to follow the selected mode.

```sh
mise exec -- python3 -m fjord export --mode day --format resolved \
  --theme examples/themes/fern.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/day-fern-compact.resolved.json
```

Copy an example theme into your own project when creating a theme. Keep its `$extensions.org.xerrion.fjord` metadata and supply complete token definitions for the paths you change. Use aliases to semantic roles when a value should adapt to each mode. `--theme` replaces the default `tokens/themes/fjord.tokens.json` layer; repeat `--override` to apply further files in order.

Use `--mode night`, `day` or `aurora`; Night is the CLI default. Each requested composition validates its own contrast, so test every mode your implementation offers. The [token contract](token-contract.md) defines allowed overrides and failure behaviour.

For an alias-preserving DTCG document for one mode, use CLI `build`. It produces the same document as `export --format dtcg`; the default filenames differ. `mise run build` calls `export` for all three modes and all three formats:

```sh
mise exec -- python3 -m fjord build --mode night \
  --theme examples/themes/soft-windows.tokens.json \
  --output dist/night-soft.tokens.json
```

## Bind to design roles

Use semantic roles such as `semantic.color.bg-base`, `semantic.color.ink` and `semantic.color.accent`. Component tokens bind those roles to design intent; their presence does not supply a component implementation.

CSS property names start with `--fjord-token-` and replace token-path dots with hyphens. For example:

```css
@import "./night.tokens.css";

body {
  background: var(--fjord-token-semantic-color-bg-base);
  color: var(--fjord-token-semantic-color-ink);
}
```

A typography composite expands into font-family, font-size, font-weight, letter-spacing and line-height properties. Shadows retain layer order; dimensions and durations retain their units. Colours become sRGB HEX, including alpha where applicable. Shared contrast checks cover original values and their rounded HEX representation. Resolved JSON preserves numeric precision and types.

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

## Migrate from contract 0.3

Use `python3 -m fjord` instead of `python3 -m fjord_tokens`, and update Python imports to `fjord`. Custom theme and override metadata must declare contract `0.4.0`; 0.3.x files are rejected until updated. Schema 2 and composition order are unchanged.

| Previous semantic role | Replacement |
| --- | --- |
| `radius.radius-<name>` | `radius.<name>` |
| `spacing.space-<number>` | `spacing.step-<number>` |
| `typography.type-<name>` | `typography.<name>` |
| `color.bg-active` | `color.bg-pressed` for a press, `color.bg-selected` for persistent neutral selection |

The old roles remain as deprecated aliases in 0.4; warnings identify their replacements. Move overrides to the replacement paths: overriding a deprecated alias does not change the new role. The new `color.accent-pressed` initially follows `accent-hover` and can be overridden independently. Re-export CSS and JSON, update bindings and verify each mode. Previous CSS property names remain available through the deprecated aliases.
