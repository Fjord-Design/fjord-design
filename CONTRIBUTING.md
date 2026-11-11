# Contributing to Fjord

Work on a feature branch, preserve unrelated work and use focused conventional commits. Run the foundation checks before committing:

```sh
mise run ci
```

Read `mise.toml` before trusting it; obtain permission before installing a missing tool or adding a dependency. Runtime code remains standard-library-only. `mise run test` runs native pytest with development-only dependencies pinned in `uv.lock`, automatically synchronizing the ignored `.venv`. See the [README test workflow](README.md#commands) for pinned tools, setup and direct runners.

## Where changes belong

| Location | Responsibility |
| --- | --- |
| `tokens/` | Design values, roles, modes and inheritance contract |
| `src/fjord/`, `tests/` | Composition, validation, export and regression coverage |
| `assets/` | Fonts, icons, brand sources, provenance and licences |
| `docs/`, `examples/` | Design rules and practical consumer guidance |
| `scripts/inspect_fonts.py` | Native inspection when replacing font assets |

Read the [token contract](docs/token-contract.md) before changing a public name, type or rule. Use the [design guidelines](docs/design-guidelines.md) for visual choices. Platform mappings and component implementations belong in separate projects.

`mise run check` runs tests and validation. `mise run build` writes three portable formats for every mode. Keep `dist/` and caches out of version control. Inspect changes and fix required checks before committing.

Keep asset bytes, manifests and original licences consistent. [Font instructions](assets/fonts/README.md) and [icon instructions](assets/icons/README.md) explain maintenance. Fjord's own source uses [MIT](LICENSE); that licence does not replace third-party asset licences.
