# Fjord

Fjord is a shared design foundation: colours, typography, layout rules, icons and fonts that separate implementations can inherit.

It has a calm Nordic character, with cool blue surfaces, readable text and restrained colour. Night, Day and Aurora provide complete modes. Theme layers and overrides adapt the design without copying the foundation.

## Use it

Run from the repository root:

```sh
mise run validate
mise run build
```

The build writes DTCG, resolved JSON and generic CSS variables for all three modes to `dist/`. Fonts, icons and brand sources remain in `assets/`, beside their licences. Only Python and its standard library are required; `mise.toml` pins the runtime. Ask before installing a missing tool.

Use the [consumer guide](docs/consumer-guide.md) to inherit or override the design. The [design guidelines](docs/design-guidelines.md) explain its visual rules, and the [token contract](docs/token-contract.md) defines names, types and composition.

## Scope

Fjord owns the shared design data, assets, inheritance and validation. Separate projects own CSS components, KDE themes, editor configurations and framework libraries, including their installation and platform tests.

```text
tokens/             Design values, modes, roles and contract
assets/             Fonts, icons, brand sources and licences
src/fjord/   Composition, validation and portable export
tests/              Foundation regression tests
docs/               Design rules and consumer contract
examples/           Theme and override files
scripts/            Font inspection for asset maintenance
dist/               Generated exports; ignored by Git
```

## Commands

| Command | Result |
| --- | --- |
| `mise run test` | Run foundation regression tests |
| `mise run validate` | Validate sources and assets |
| `mise run check` | Run tests and validation |
| `mise run build` | Export every mode as DTCG, resolved JSON and CSS |
| `mise run ci` | Run checks and build |

The CLI provides `validate`, `build` and `export`. CLI `build` writes DTCG for one mode; `mise run build` exports all three formats for all modes. Font and icon checks run with `validate`, not token-only CLI builds. For example:

```sh
mise exec -- python3 -m fjord export --mode day --format resolved \
  --theme examples/themes/fern.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/day-fern-compact.resolved.json
```

The previous component library, adapters, previews and release tooling remain available in [Git history](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb). They are outside the current project scope. See the [scope decision](docs/decisions/0002-shared-design-foundation.md), [breaking-change notes](CHANGELOG.md) and [contribution rules](CONTRIBUTING.md).
