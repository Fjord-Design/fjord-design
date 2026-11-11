# Changelog

## Unreleased: contract 0.5 breaking cutover

This section supersedes the 0.4 compatibility guidance in the historical entries below.

- Bumped the source contract and all source/example layer metadata to `0.5.0`. Only compatible `0.5.x` layers are accepted; `0.4.x` and earlier layers are rejected. Schema `2`, DTCG `2025.10` and composition order are unchanged.
- Removed all 32 deprecated semantic roles: ten `spacing.space-*` roles in favour of `spacing.step-*`, six `radius.radius-*` roles in favour of `radius.*`, fifteen `typography.type-*` roles in favour of `typography.*`, and `color.bg-active` in favour of `color.bg-pressed` for a press or `color.bg-selected` for persistent neutral selection. These paths are all within the `semantic` namespace; their aliases, CSS properties and JSON paths are no longer exported.
- Renamed `primitive.color.{night,day,aurora}.bg-active` to `bg-pressed` without changing their values. Updated references and descriptions; modes now provide 87 semantic colour roles instead of 88.
- Retained current design values, all three default mode appearances and the five Fern, Water, Mountain, Cabin and Hearth accent themes.
- Moved the five official Fern, Water, Mountain, Cabin and Hearth accent themes from `examples/themes/` to `tokens/themes/`; select them with `--theme tokens/themes/<name>.tokens.json`. The old file paths are removed, not aliases. The `gaming`, `work` and `soft-windows` customization examples remain under `examples/themes/`. Theme contents, the default `tokens/themes/fjord.tokens.json` layer, contract `0.5.0` and schema `2` are unchanged.
- Retained generic DTCG `$deprecated` metadata validation and use-based CLI warnings for future deprecations. No definitions in shipped sources, the default theme or example layers are currently deprecated.
- See the [0.5 migration guide](docs/consumer-guide.md#migrate-to-contract-05) for complete path mappings, layer metadata, consumer bindings and the earlier Python module rename.
- Migrated regression tests from unittest to native pytest, with development-only dependencies locked by uv and Python/uv pinned through mise. `mise run test` now provisions the isolated `.venv` and runs pytest; `mise run ci` uses the same locked test environment. The foundation runtime remains standard-library-only.
- Added local-document JSON Pointer `$ref` resolution for complete token values, nested properties, array elements and metadata, with RFC 6901 escaping, URI-fragment decoding and errors for malformed paths, missing targets, cycles and type mismatches. External files and network references remain unsupported.
- Added alias-derived token types, reserved group `$root` tokens and local `$extends` group inheritance. Nested groups merge, while local token definitions replace inherited tokens in full; missing, token-valued and circular inheritance targets fail.
- Preserved unknown root, group and token `$extensions` through composition and DTCG export. Generated Fjord metadata is merged without discarding the original design name, design seeds, asset manifests or unrelated vendor metadata.
- Corrected shadow-array aliases: each element reference must resolve to a single shadow object. References to arrays and literal nested arrays are rejected instead of flattened; whole-token aliases to shadow arrays remain valid.
- Distinguished authored composition fragments from self-contained DTCG exports and documented the repaired local reference behavior. Contract `0.5.0`, schema `2`, existing design values, accent/status behavior and intentional supported-type, sRGB and project-namespace limits are unchanged; these repairs do not claim universal DTCG conformance.

## Unreleased: contract 0.4 naming migration

- Renamed the Python module from `fjord_tokens` to `fjord`; update imports and commands.
- Added distinct neutral pressed and selected colours and an independently overridable `accent-pressed` role.
- Added shorter semantic radius, spacing and typography paths. Previous paths and `bg-active` remain deprecated aliases in 0.4.
- CLI `build` and `export --format dtcg` now emit identical metadata; `mise run build` exports all formats and modes. CLI success messages begin with `Wrote`.
- Theme and override metadata must declare contract 0.4.x. See the [migration guide](docs/consumer-guide.md#migrate-to-contract-05).
- Fixed deprecation warnings to identify actual deprecated aliases and theme/override assignments, including the requested build/export composition, instead of listing every retained definition. Deprecated 0.4 aliases remain supported.
- Updated the compact-spacing and soft-windows examples to override `semantic.spacing.step-4` and `semantic.radius.window`, so their adjustments reach modern bindings.
- Added Water (blue), Mountain (violet), Cabin (wood brown) and Hearth (ember-orange) alongside the existing Fern (green) accent theme for every mode through `--theme examples/themes/{name}.tokens.json`. Default mode appearances and status, syntax, ANSI and chart colours are unchanged.
- Added primitive-backed `hearth`, `hearth-hover`, `hearth-soft` and `lilac-soft` roles in every mode, increasing semantic colour roles from 84 to 88 within contract 0.4.x.

## Earlier unreleased scope reduction

This is a breaking reduction of the project's delivery scope. The existing token names, values, schema `2`, contract `0.3.0` and `base → mode → theme → overrides` composition are retained.

- Removed component libraries, platform adapters and their application dependencies from the active project.
- Removed previews, Storybook, catalogues, native packages, release archives and their commands.
- Limited the CLI to `validate`, `build` and `export`, with `dtcg`, `resolved` and `css` export formats. Native WezTerm export and `theme-css` are removed.
- Kept the shared design rules, font/icon/brand assets, theme examples, portable exports and validation.

Consumers using removed commands or packages must keep the previous source revision while extracting their implementation into a separate project. New consumers use the [consumer guide](docs/consumer-guide.md).

## Historical combined releases

The owner accepted the combined `1.0.0` release for local distribution on 8 November 2026, and the earlier MVP `0.1.0` on 29 October 2026. Their acceptance and recorded verification limits are unchanged. Their original [changelog](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/CHANGELOG.md), [source](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb) and [verification records](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/verification) remain in Git history. This unreleased scope change does not replace those release artifacts or claim a new acceptance.
