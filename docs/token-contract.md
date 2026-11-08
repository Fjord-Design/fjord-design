# Token contract

Use `mise run check` before accepting a token change. The source contract is `tokens/contract.json`, schema version 2, contract version 0.3.0. The `*.tokens.json` files use the [DTCG Format Module 2025.10](https://www.designtokens.org/tr/2025.10/format/).

## Names and ownership

| Namespace | Meaning | Example |
| --- | --- | --- |
| `primitive` | Typed source values, with palettes grouped by mode | `primitive.color.night.accent` |
| `semantic` | Portable roles consumed by themes and adapters | `semantic.color.accent` |
| `component` | Component bindings to semantic roles | `component.button.primary.background` |

Names use lower-case letters, digits and hyphens inside dot-separated group paths. Keep application-specific names and mappings in separate implementation projects. Every token has a type and a non-empty description. Groups may supply inherited `$type` and `$deprecated` metadata. A token cannot also contain child groups.

Primitives can reference other primitives. Semantic roles can reference primitives or other semantic roles. Every component token must be a whole-token alias directly to a semantic role. Literal colours, dimensions and primitive aliases in component tokens fail validation.

```json
{
  "component": {
    "button": {
      "primary": {
        "background": {
          "$type": "color",
          "$value": "{semantic.color.accent}",
          "$description": "Primary button background."
        }
      }
    }
  }
}
```

## Format profile

Fjord validates the subset of DTCG used by its sources. Supported types are `color`, `dimension`, `duration`, `number`, `fontFamily`, `fontWeight`, `cubicBezier`, `shadow` and `typography`. Source documents and built documents preserve DTCG aliases. This tool is not a validator for every feature of DTCG.

| Type | Fjord representation |
| --- | --- |
| Color | `colorSpace: "srgb"`, three numeric components from 0 to 1 and optional numeric `alpha` |
| Dimension | Numeric `value`, with `px` or `rem` |
| Duration | Non-negative numeric `value`, with `ms` or `s` |
| Number | Finite JSON number; used for stacking order |
| Font family | A family string or a non-empty fallback array |
| Font weight | Numeric value from 1 to 1000 |
| Cubic Bézier | Four finite numbers; x coordinates between 0 and 1 |
| Shadow | Typed colour, offsets, blur and spread, optional boolean `inset`; arrays preserve layer order |
| Typography | Typed font family, size, weight, letter spacing and numeric line-height ratio |

Letter spacing uses pixels, and line height is a ratio of the font size. `fontStyle` has no field in the supported DTCG typography composite; Fjord declares it in the token's `$extensions.org.xerrion.fjord.fontStyle`, alongside its text sample. Base metadata holds the design name, `designSeeds` and the `fontAssets`/`iconAssets` manifest references. The repository includes the pinned assets with original licences. The directory contract separately identifies `usage.json` for semantic meanings, states and pair thresholds.

Semantic spacing, radii and border widths must be non-negative. Semantic font sizes must be positive, and stacking order must be an integer. These role constraints are checked after each layer, including a layer later overridden by another file.

Aliases use complete `{token.path}` references, including typed fields inside composites. Missing references, cycles and type mismatches fail. Unsupported `$ref` JSON Pointers, `$extends`, additional colour spaces and other DTCG types are rejected with an error. No external schema or remote data is fetched during validation.

## Composition

| Order | Layer | Contract |
| --- | --- | --- |
| 1 | `base.tokens.json` | Primitives, shared semantic tokens and component bindings |
| 2 | Exactly one `modes/<mode>.tokens.json` | All 81 semantic colour roles and four semantic shadow roles |
| 3 | Default `themes/fjord.tokens.json`, or explicit `--theme` | Optional identity changes at existing semantic or component paths |
| 4 | Repeated `--override` files, left to right | Allowed local changes; the last supplied token definition wins |

Night is selected only when the CLI receives no mode. A mode cannot inherit missing values from Night or from base. Base is forbidden from defining mode-specific semantic colour or shadow roles. Each mode must supply its complete set, even when some values are shared through explicit aliases.

Override rules:

1. Replace a complete token definition, including its `$type`, `$value` and `$description`. Objects and arrays are replaced atomically. A partial shadow or typography object cannot merge with an earlier value.
2. Override only existing `semantic` or `component` paths. Primitive changes belong in the source palette. Add new roles to the source contract before referencing them from an override.
3. Preserve the token's type. Unknown paths, `null` deletion markers and malformed values fail. An invalid layer cannot be hidden by a later layer.
4. Resolve aliases against the composed graph after each layer. Component and derived semantic values follow the final alias target. The Python composition result records the winning layer in `provenance`.
5. Declare schema version 2 and a compatible `0.3.x` contract in `$extensions.org.xerrion.fjord`. See the example files for complete metadata. Missing or incompatible metadata fails.

The selected mode's final contrast is checked after composition. `validate` checks the default modes and shared assets. `build` first runs that default project validation, then checks the requested theme and overrides for the selected mode. A Night-only colour override is not automatically approved for Day or Aurora.

## Validation and output

The validator rejects duplicate JSON keys, non-finite numbers, invalid names, missing descriptions, unsupported types, missing required roles and malformed typed values. Required roles are declared in the source contract, with baseline roles protected in `src/fjord_tokens/contract.py`. New custom groups or app-specific token namespaces require an explicit contract change.

Contrast uses unrounded sRGB components. Text requires 4.5:1; the specified focus and control-border pairs require 3:1. Comments on `bg-base` require 5:1, increased to 7.5:1 in Night. The primary button's default, hover, pressed and focus bindings are checked after overrides. Editor, terminal, Git, diagnostic and diff bindings add checks for their actual surfaces. Alpha is rejected in checked foreground/background pairs. The translucent scrim and shadow layers remain supported outside those pairs. ANSI black has a background role in dark modes; its text contrast is checked in Day. Disabled text and decorative dividers are excluded.

The same pairs must also pass after conversion to the final 8-bit HEX components. Ratios remain unrounded at the comparison. An optional colour `hex` hint must match the rounded RGB components. The seven original OKLCH inspiration values are validated as metadata. Canonical colour values are sRGB; the original design-seed metadata does not introduce another output colour space.

The semantic usage source contributes its documented UI/state pairs to the same checks. Duplicate pairs retain the strongest threshold. Loading text keeps normal contrast; disabled foreground is restricted to unavailable controls and cannot be rebound into help or active text. A theme must retain component token paths, types and aliases. Check each mode your implementation exposes; the [consumer guide](consumer-guide.md) shows composition and export.

The seven main syntax colours must differ by at least 7 in OKLab distance multiplied by 100. Pairs involving operator or regex colours require at least 5. These are Fjord design thresholds, independent of text contrast. The checks run after overrides. See [code and terminal rules](design-guidelines.md#code-and-terminal) for bindings and examples.

Font and icon validation checks the retained manifests, exact asset bytes and original licences. Platform mappings and native output checks belong to consuming implementations; token validation does not read platform documentation.

Builds preserve aliases and include all referenced tokens in a single DTCG document per mode. They validate before writing and replace the generated file atomically. Failed builds preserve an existing output file. Build output cannot replace files under `tokens/` or supplied theme/override files.

`dist/` is ignored by Git. Review source changes and regenerate exports. Production components, font loading, platform APIs and accessibility testing remain responsibilities of the consuming implementation.

## Compatibility

Source layers declare DTCG `2025.10`, schema `2` and a supported `0.3.x` token contract. Token paths, types, alias rules and merge order are public data contracts. A breaking token change requires a new minor version while the contract is below `1.0`, with affected paths and required consumer changes documented. Use `$deprecated` with a replacement path before removing a role, keeping its type and alias valid throughout the supported contract family.

The foundation-only scope retains this serialized token contract. Removed CLI commands and implementation packages are a separate breaking delivery change recorded in the [changelog](../CHANGELOG.md).
