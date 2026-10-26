# Versions and token lifecycle

Use `mise run check` and `mise run build` before accepting a contract change.

| Version | Current value | Purpose |
| --- | --- | --- |
| DTCG | `2025.10` | Portable token serialization |
| Fjord schema | `2` | Directory manifest and layer metadata |
| Fjord contract | `0.3.0` | Public names, types, roles and CLI behaviour |

The resolver accepts `0.3.x` contracts. Every source and override must declare schema version `2` and DTCG version `2025.10`. Unsupported versions fail before composition or output.

The [dependency policy](dependency-policy.md) governs library and development-tool releases, including the selected Svelte reference and its TypeScript 7 support arrangement. Dependency versions are independent of token-contract versions.

Contract 0.3.0 includes the layout roles, semantic usage/state source, semantic CLI, System preference and scoped identity contract. All source/override metadata must declare a supported 0.3.x contract. `tokens/contract.json` identifies `usage.json`; consumers import the scoped semantic stylesheet and may choose `mode="system"`. Existing token names and merge order remain unchanged. The Svelte package is versioned independently; 0.6.0 adds Dialog, Alert and Tooltip.

## Release rules

During `0.x`, changes to public names, types, required roles, merge precedence, CLI arguments or output semantics require a new minor contract version. After `1.0`, breaking changes require a new major version. Patches may correct values within existing roles or improve documentation and diagnostics. Every release must pass value, alias, contrast and platform-table checks.

For a breaking release, document affected token paths, replacement paths and types, the supported contract version, and the consumer changes required. Verify consumer bindings and resolved values before adopting the release. Token sources belong in version control; generated output belongs in `dist/`.

## Token lifecycle

Use `$deprecated` to mark a role scheduled for removal and name its replacement. Keep its type and alias target valid throughout the supported contract family. Remove the role only in a breaking release, after checking its consumers.

Deprecation is inherited from groups and can be cleared with `false` on a child. Validation reports deprecated tokens as warnings on stderr and continues checking their values, references and contrast. The current sources contain no deprecated tokens.

Overrides cannot add token paths. Introduce a replacement role in the source contract before using it in an override or component binding.
