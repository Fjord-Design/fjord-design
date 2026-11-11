# Token contract

Use `mise run check` before accepting a token change. The source contract is `tokens/contract.json`, schema version 2, contract version 0.5.0. The `*.tokens.json` files use Fjord's profile of the [DTCG Format Module 2025.10](https://www.designtokens.org/tr/2025.10/format/).

## Names and ownership

| Namespace | Meaning | Example |
| --- | --- | --- |
| `primitive` | Typed source values, with palettes grouped by mode | `primitive.color.night.accent` |
| `semantic` | Portable roles consumed by themes and adapters | `semantic.color.accent` |
| `component` | Component bindings to semantic roles | `component.button.primary.background` |

Ordinary names use lower-case letters, digits and hyphens inside dot-separated group paths. The reserved token name `$root` is also allowed inside groups and remains explicit in token paths. Keep application-specific names and mappings in separate implementation projects. Every token must have an effective type and a non-empty description; the type may be declared, derived from a whole-token alias or inherited from a group. Groups may supply inherited `$type` and `$deprecated` metadata. A token cannot also contain child groups.

Primitives can reference other primitives. Semantic roles can reference primitives or other semantic roles. Every component token must be a whole-token alias directly to a semantic role, using either `{semantic.path}` or a local `$ref` to that role's complete `$value`. Literal colours, dimensions, primitive aliases and property-only references in component tokens fail validation. Both reference syntaxes obey the same namespace and type constraints.

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

Fjord validates the DTCG features used by its sources and the local-document reference and group behavior described below. Supported types are `color`, `dimension`, `duration`, `number`, `fontFamily`, `fontWeight`, `cubicBezier`, `shadow` and `typography`. DTCG exports preserve value aliases and metadata. This is a constrained project profile, not a claim of universal DTCG conformance: other token types and colour spaces remain unsupported, and Fjord's naming, description, namespace and role policies still apply.

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

Unknown vendor `$extensions` are preserved at the document root, on groups and on tokens, including through flattening/nesting and DTCG export. Generated Fjord export metadata is merged into `$extensions.org.xerrion.fjord`, retaining the original design name, `designSeeds`, asset manifest references and unrelated extension data. See DTCG [§5.2.3](https://www.designtokens.org/tr/2025.10/format/#extensions) and [§6.3.2](https://www.designtokens.org/tr/2025.10/format/#extensions-0).

Semantic spacing, radii and border widths must be non-negative. Semantic font sizes must be positive, and stacking order must be an integer. These role constraints are checked after each layer, including a layer later overridden by another file.

### References and types

Use `{token.path}` for a complete token value, including references in typed composite fields. For example, `{semantic.spacing.step-2}` always selects that token's `$value`, not a property within it. A local JSON Pointer uses an object such as `{"$ref": "#/semantic/spacing/step-2/$value/value"}` to select an exact document location. Pointers can select whole `$value` objects, nested properties, array elements or metadata; the resulting value must match the type expected at the use site. See DTCG [§6.6](https://www.designtokens.org/tr/2025.10/format/#references-and-json-pointer-integration) and [§7.1–7.4](https://www.designtokens.org/tr/2025.10/format/#reference-syntax).

This self-contained format example reuses the numeric part of a dimension. It illustrates parsing, not a complete Fjord project with all required roles and project metadata:

```json
{
  "primitive": {
    "spacing": {
      "$type": "dimension",
      "step": {
        "$value": {"value": 8, "unit": "px"},
        "$description": "Base spacing step."
      },
      "copy": {
        "$value": {
          "value": {"$ref": "#/primitive/spacing/step/$value/value"},
          "unit": "px"
        },
        "$description": "Spacing using the base step's numeric value."
      }
    }
  }
}
```

Pointers follow [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901): use `~0` for `~` and `~1` for `/` within a path segment, URI-fragment decoding for encoded characters, and zero-based indices when traversing arrays. Missing targets, malformed paths, cycles and type mismatches fail. References must stay in the local composed document; external files, URLs, remote schemas and network fetching are not supported.

A token's explicit `$type` takes precedence. Without it, a whole-token alias derives its resolved target's type; otherwise the nearest containing group's `$type`, including one inherited through `$extends`, supplies the type. A token with no determinable type fails: Fjord does not guess types from arbitrary literal values or property-reference results. See DTCG [§5.2.2](https://www.designtokens.org/tr/2025.10/format/#type-0) and [§6.7](https://www.designtokens.org/tr/2025.10/format/#processing-rules).

### Groups and `$root`

A group can contain a `$root` token alongside variants. Reference it explicitly as `{group.$root}` or with a pointer ending in `/$root/$value`; a reference to the group itself is not a token alias. `$extends` accepts a brace reference to a group or a local JSON Pointer to that group. It inherits tokens, nested groups and group properties, including `$type`, `$deprecated` and `$extensions`. Nested groups merge, but a locally defined token replaces the entire inherited token definition. Missing targets, token targets and inheritance cycles fail. See DTCG [§6.1–6.4](https://www.designtokens.org/tr/2025.10/format/#groups).

In this format example, `compact` replaces the root token and inherits `wide`. `"$extends": "#/primitive/spacing/regular"` is the equivalent pointer spelling:

```json
{
  "primitive": {
    "spacing": {
      "$type": "dimension",
      "regular": {
        "$root": {
          "$value": {"value": 8, "unit": "px"},
          "$description": "Regular spacing."
        },
        "wide": {
          "$value": {"value": 16, "unit": "px"},
          "$description": "Wide spacing."
        }
      },
      "compact": {
        "$extends": "{primitive.spacing.regular}",
        "$root": {
          "$value": {"value": 4, "unit": "px"},
          "$description": "Compact spacing."
        }
      }
    }
  }
}
```

The compact root value is `{primitive.spacing.compact.$root}` or `{"$ref": "#/primitive/spacing/compact/$root/$value"}`. These format features do not add roles to the Fjord contract or permit a theme to introduce new token paths.

### Shadow arrays

A shadow value can be one shadow object or an ordered array of shadow objects and references. Each array-element reference must resolve to a **single shadow object**. Literal nested arrays and element references to shadow arrays are invalid; arrays are never expanded or flattened. A whole-token alias to an entire shadow array remains valid. See DTCG [§9.1](https://www.designtokens.org/tr/2025.10/format/#array-aliasing-in-composite-types) and [§9.6](https://www.designtokens.org/tr/2025.10/format/#shadow).

## Composition

Authored base, mode, theme and override files are composition fragments. A fragment may reference tokens supplied by other layers and is not necessarily a self-contained DTCG interchange document. Fjord composes the hierarchical documents before flattening and resolving values: group trees and their metadata merge, while token definitions replace atomically. The built DTCG document is self-contained for the selected mode and includes the reference targets needed by its preserved aliases.

| Order | Layer | Contract |
| --- | --- | --- |
| 1 | `tokens/base.tokens.json` | Primitives, shared semantic tokens and component bindings |
| 2 | Exactly one `tokens/modes/<mode>.tokens.json` | All 87 semantic colour roles and four semantic shadow roles |
| 3 | Default `tokens/themes/fjord.tokens.json`, or explicit `--theme` | Optional theme changes at existing semantic or component paths |
| 4 | Repeated `--override` files, left to right | Allowed local changes; the last supplied token definition wins |

Night is selected only when the CLI receives no mode. A mode cannot inherit missing values from Night or from base. Base is forbidden from defining mode-specific semantic colour or shadow roles. Each mode must supply its complete set, even when some values are shared through explicit aliases.

The `hearth`, `hearth-hover`, `hearth-soft` and `lilac-soft` colour roles remain primitive-backed in every mode. They supply the Hearth ember-orange family and the Mountain soft selection fill without changing existing role values. Current source and example metadata declares schema `2`, contract `0.5.0` and DTCG `2025.10`.

The [selectable accent themes](consumer-guide.md#choose-an-accent) use `examples/themes/{name}.tokens.json` for `fern`, `water`, `mountain`, `cabin` and `hearth`, independently of mode. They alias `accent`, `accent-hover`, `accent-soft`, `accent-ink` and `selection`; `accent-pressed` still follows `accent-hover`, `focus-ring` follows `accent`, and `on-accent` remains mode-provided. They do not rewire status, syntax, ANSI or chart roles. Omitting `--theme` preserves each mode's default appearance.

Override rules:

1. Replace a complete token definition, including `$value` and `$description`, with an explicit `$type` unless an alias or group supplies its effective type. Tokens, including composite objects and arrays in their values, are replaced atomically. Group containers merge; a partial shadow or typography object cannot merge with an earlier token value.
2. Override only existing `semantic` or `component` paths. Primitive changes belong in the source palette. Add new roles to the source contract before referencing them from an override.
3. Preserve the token's type. Unknown paths, `null` deletion markers and malformed values fail. An invalid layer cannot be hidden by a later layer.
4. Resolve aliases against the composed graph after base and mode are combined, then after each theme or override. An alias in a fragment can use a target from the composed document; it does not fetch another file. Component and derived semantic values follow the final alias target, including an inferred type when `$type` is omitted. The Python composition result records in `provenance` the layer supplying each token definition or group inheritance declaration. Later changes to an alias or inheritance target update derived values without changing the dependent token's provenance.
5. Declare schema version 2 and a compatible `0.5.x` contract in `$extensions.org.xerrion.fjord`; current sources and examples use `0.5.0`. Missing or incompatible metadata fails. In particular, `0.4.x` and earlier layers are rejected rather than reinterpreted under the new contract.

The selected mode's final contrast is checked after composition. `validate` checks the default modes and shared assets. CLI `build` and `export` validate the default token project and the requested composition; they do not validate font or icon files. Run `mise run validate` to check assets. A Night-only colour override is not automatically approved for Day or Aurora.

## Validation and output

The validator rejects duplicate JSON keys, non-finite numbers, invalid names, missing descriptions, unsupported types, missing required roles and malformed typed values. Required roles are declared in the source contract, with baseline roles protected in `src/fjord/contract.py`. New custom groups or app-specific token namespaces require an explicit contract change.

No definitions in Fjord's shipped sources, default theme or example layers are currently deprecated. Generic DTCG `$deprecated` metadata validation and use-based diagnostics remain supported for future deprecations. Diagnostics report actual use in supplied token inputs: aliases to deprecated roles, including references inside composite values, and explicit theme or override assignments to deprecated paths, even if the new definition omits or replaces `$deprecated`. Each applied layer is checked, including one later overridden. Warnings identify the source, deprecated role and replacement. Deprecated definitions alone do not produce warnings during validation, builds or exports. The CLI cannot inspect external consumers for deprecated or removed CSS properties or JSON paths.

Contrast uses unrounded sRGB components. Text requires 4.5:1; the specified focus and control-border pairs require 3:1. Comments on `bg-base` require 5:1, increased to 7.5:1 in Night. The primary button's default, hover, pressed and focus bindings are checked after overrides. Editor, terminal, Git, diagnostic and diff bindings add checks for their actual surfaces. Alpha is rejected in checked foreground/background pairs. The translucent scrim and shadow layers remain supported outside those pairs. ANSI black has a background role in dark modes; its text contrast is checked in Day. Disabled text and decorative dividers are excluded.

The same pairs must also pass after conversion to the final 8-bit HEX components. Ratios remain unrounded at the comparison. An optional colour `hex` hint must match the rounded RGB components. The seven original OKLCH inspiration values are validated as metadata. Canonical colour values are sRGB; the original design-seed metadata does not introduce another output colour space.

The semantic usage source contributes its documented UI/state pairs to the same checks. Duplicate pairs retain the strongest threshold. Loading text keeps normal contrast; disabled foreground is restricted to unavailable controls and cannot be rebound into help or active text. A theme must retain component token paths, types and aliases. Check each mode your implementation exposes; the [consumer guide](consumer-guide.md) shows composition and export.

The seven main syntax colours must differ by at least 7 in OKLab distance multiplied by 100. Pairs involving operator or regex colours require at least 5. These are Fjord design thresholds, independent of text contrast. The checks run after overrides. See [code and terminal rules](design-guidelines.md#code-and-terminal) for bindings and examples.

Font and icon validation checks the retained manifests, exact asset bytes and original licences. Platform mappings and native output checks belong to consuming implementations; token validation does not read platform documentation.

Builds preserve aliases and include all referenced tokens in a single DTCG document per mode. They validate before writing and replace the generated file atomically. Failed builds preserve an existing output file. Build output cannot replace files under `tokens/` or supplied theme/override files.

`dist/` is ignored by Git. Review source changes and regenerate exports. Production components, font loading, platform APIs and accessibility testing remain responsibilities of the consuming implementation.

## Compatibility

Source layers declare DTCG `2025.10`, schema `2` and a supported `0.5.x` token contract. Token paths, types, alias rules and merge order are public data contracts. A breaking token change requires a new minor version while the contract is below `1.0`, with affected paths and required consumer changes documented. For a future deprecation, use `$deprecated` with a replacement path before removing a role, keeping its type and alias valid throughout the supported contract family.

Contract 0.5 removes all 32 deprecated semantic definitions: ten `spacing.space-*` roles, six `radius.radius-*` roles, fifteen `typography.type-*` roles and `color.bg-active`. Their aliases, CSS properties and JSON paths are no longer exported. The three `primitive.color.<mode>.bg-active` paths are also renamed to `bg-pressed`, retaining their original values. Current design values, default mode appearances, accent themes, schema and DTCG version are unchanged. Follow the [0.5 migration guide](consumer-guide.md#migrate-to-contract-05) for the complete old-to-new mappings and the distinction between pressed and selected backgrounds.
