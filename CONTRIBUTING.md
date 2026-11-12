# Contributing to Fjord

Work on a feature branch, preserve unrelated work and use focused conventional commits. Run the foundation checks before committing:

```sh
mise run ci
```

Read `mise.toml` before trusting it; obtain permission before installing a missing tool or adding a dependency. Runtime code is a Rust crate; dependencies are locked in `Cargo.lock`. `mise run test` runs `cargo test --locked` and `mise run lint` runs rustfmt and clippy. See the [README commands](README.md#commands) for setup.

## Where changes belong

| Location | Responsibility |
| --- | --- |
| `tokens/` | Design values, roles, modes and inheritance contract |
| `src/`, `tests/` | Composition, validation, export and regression coverage |
| `assets/` | Fonts, icons, brand sources, provenance and licences |
| `docs/`, `examples/` | Design rules and practical consumer guidance |
| `scripts/inspect_fonts.py` | Native inspection when replacing font assets |

Read the [token contract](docs/token-contract.md) before changing a public name, type or rule. Use the [design guidelines](docs/design-guidelines.md) for visual choices. Platform mappings and component implementations belong in separate projects.

`mise run check` runs lint, tests and validation. `mise run build` writes three portable formats for every mode. Keep `dist/` and caches out of version control. Inspect changes and fix required checks before committing.

Keep asset bytes, manifests and original licences consistent. [Font instructions](assets/fonts/README.md) and [icon instructions](assets/icons/README.md) explain maintenance. Fjord's own source uses [MIT](LICENSE); that licence does not replace third-party asset licences.
