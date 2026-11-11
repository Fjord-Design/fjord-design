# Fjord

Fjord is a shared design foundation: colours, typography, layout rules, fonts and icons that separate implementations can inherit. Its character is calm and Nordic, with cool blue surfaces, readable text and restrained colour.

## Get started

Run all commands from the `fjord-design` checkout root, the directory containing `mise.toml`, `tokens/`, `src/` and `assets/`.

Use [mise](https://mise.jdx.dev/) to run the project tasks. `mise.toml` pins Python 3.15.0 and uv 0.13.7. The foundation CLI and validation use only Python's standard library; pytest and its dependencies are development-only. Review `mise.toml` before trusting it, and obtain permission before installing a missing tool.

```sh
mise run validate
mise run build
```

Validation checks the default token project, fonts, icons and pinned licences. On success, it prints a `PASS:` summary of tokens, modes, contrast pairs, font faces and icons.

The build writes nine files, one per mode and format, and prints `Wrote <mode>: <output path>` for each export:

| Format | Night file | Contents |
| --- | --- | --- |
| DTCG | `dist/night.dtcg.json` | Typed tokens with aliases and metadata |
| Resolved JSON | `dist/night.resolved.json` | Flat token paths with resolved and original values, types and source provenance |
| CSS | `dist/night.tokens.css` | Generic CSS custom properties |

For Day and Aurora, replace `night` in each filename with `day` or `aurora`. `dist/` is generated and Git-ignored. Fonts, icons and brand sources are not copied there; they remain in `assets/` with their licences.

## Use an export

A token is a named design value. Use semantic roles such as `semantic.color.bg-base`, `semantic.color.ink` and `semantic.color.accent` rather than copying palette values. Resolved JSON consumers can read the public `tokens` mapping and its `.value` entries.

For example, with `night.tokens.css` next to your stylesheet:

```css
@import "./night.tokens.css";

body {
  background: var(--fjord-token-semantic-color-bg-base);
  color: var(--fjord-token-semantic-color-ink);
}
```

The CSS variables are portable, not a component library. The consuming project chooses their scope and handles runtime mode switching.

Night is the default, with dark blue surfaces and a frost-blue accent. Day uses pale blue-grey surfaces and a deep-blue accent; Aurora uses almost-black blue and teal. All three modes provide complete values for the same token names. Theme layers and overrides adapt the foundation without copying it: composition goes from base to one mode, a theme and then ordered overrides.

Official accent themes live in `tokens/themes/` alongside the default `fjord.tokens.json`. They are independent of mode: Fern (green), Water (blue), Mountain (violet), Cabin (wood brown) and Hearth (ember-orange) work with `night`, `day` or `aurora`. Select one with `--theme tokens/themes/{name}.tokens.json`, using the lower-case accent name. Without `--theme`, all default mode appearances stay unchanged. These themes change interaction and selection colours, not status, syntax, ANSI or chart colours.

The `gaming`, `work` and `soft-windows` themes under `examples/themes/` are customization examples, not official accent themes.

The [consumer guide](docs/consumer-guide.md) explains implementation and overrides, the [design guidelines](docs/design-guidelines.md) describe visual intent, and the [token contract](docs/token-contract.md) defines names, types and composition.

The current token contract is `0.5.0`; source, theme and override layers must declare a compatible `0.5.x` version. Schema `2` and DTCG `2025.10` are unchanged. Contract 0.5 removes the 32 deprecated semantic roles and renames the primitive pressed-background paths without changing current design values. Old aliases, CSS properties and JSON paths are no longer exported. Follow the [0.5 migration guide](docs/consumer-guide.md#migrate-to-contract-05) before upgrading an existing consumer.

Fjord supports local DTCG JSON Pointer `$ref` values, including property references, group `$extends`, explicit `$root` tokens and types derived from whole-token aliases. DTCG exports are self-contained and preserve root, group and token extension data; authored project layers may be composition fragments whose references resolve only after layering. The [format profile](docs/token-contract.md#format-profile) documents examples and limits: supported token types, sRGB colours and Fjord's naming, namespace and role policies still apply. External references are not fetched, and Fjord does not claim universal DTCG conformance.

## Commands

| Command | Result |
| --- | --- |
| `mise run test` | Sync locked development dependencies and run native pytest regression tests |
| `mise run validate` | Validate sources and assets |
| `mise run check` | Run tests and validation |
| `mise run build` | Export every mode as DTCG, resolved JSON and CSS |
| `mise run ci` | Sync test dependencies, run checks and build |

`mise run test` uses native pytest through uv. The equivalent direct command is `mise exec -- uv run --locked --python python3 -m pytest`; mise supplies the pinned Python, uv discovery policy and absolute `PYTHONPATH` needed by CLI subprocess tests. uv creates or synchronizes the ignored local `.venv` from `uv.lock`, including pytest and its development dependencies. No runtime package is built or installed. To prepare that environment without running tests:

```sh
mise exec -- uv sync --locked --python python3
```

On a clean checkout, review `mise.toml`, run `mise trust`, then use `mise install` to install any missing pinned tools with approval. `UV_PYTHON_PREFERENCE=only-system` keeps uv on mise's Python instead of selecting a uv-managed interpreter. The GitHub Actions workflow installs the pinned tools with mise and runs `mise run ci`; the test task provisions its own locked dependencies.

The CLI provides `validate`, `build` and `export`. CLI `build` writes DTCG for one mode; `mise run build` exports all three formats for all modes. Font and icon checks run with `validate`, not token-only builds.

For example, this exports Day with the Fern theme and the supplied local override:

```sh
mise exec -- python3 -m fjord export --mode day --format resolved \
  --theme tokens/themes/fern.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/day-fern-compact.resolved.json
```

## Assets and scope

The [fonts](assets/fonts/README.md) include their original OFL notices; the [icons](assets/icons/README.md) are Phosphor Fill SVGs with an MIT notice; and the [brand guide](assets/brand/README.md) covers the brand sources. Preserve relevant licences, manifests and provenance when reusing assets. Repository source uses the MIT licence in [LICENSE](LICENSE), which does not replace third-party asset licences.

Fjord owns shared design data and rules, assets, inheritance, portable exports and validation. Separate projects own CSS components, KDE themes, editor configurations and framework libraries, including installation, rendering and platform and accessibility tests.

```text
tokens/      Design values, modes, built-in themes, roles and contract
assets/      Fonts, icons, brand sources and licences
src/fjord/   Composition, validation and portable export
tests/       Foundation regression tests
docs/        Design rules and consumer contract
examples/    Custom theme and override examples
scripts/     Font inspection for asset maintenance
dist/        Generated exports; ignored by Git
```

The previous component library, adapters, previews and release tooling remain in [Git history](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb), outside the current project scope. See the [scope decision](docs/decisions/0002-shared-design-foundation.md) and [breaking-change notes](CHANGELOG.md).

For contributions, follow the [contribution rules](CONTRIBUTING.md), run `mise run ci` before committing, and keep generated `dist/` files and caches out of Git.
