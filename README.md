# Fjord

Fjord is a digital place with room to think. It is a calm Nordic design system for the tools and spaces we use throughout the day, from websites and applications to editors and terminals.

Cool blue surfaces, readable text and restrained colour give Fjord a familiar character. Small warm accents make the space feel personal. Motion has a purpose, and status carries meaning through text or icons as well as colour.

Fjord offers three modes for different moments:

- **Night** is the default, with a dark blue ground and frost blue accents.
- **Day** uses a pale blue grey ground and deeper colours for daylight.
- **Aurora** uses a deeper blue ground and teal accents for late nights and gaming.

The same design roles and principles guide each mode. Fjord is intended for designers and developers who want a consistent visual language across the things they build and use. Each platform needs its own adapter and verification before support can be claimed.

## What is in this repository

The DTCG token sources define colour, typography and component roles. A Python core composes themes and overrides and validates their values and contrast. The repository also contains design guidance, a code and terminal specimen, and a Svelte reference library with Storybook.

The Svelte reference now implements the exact 66-component source inventory. The complete-library candidate adds 43 components, typed APIs, family specifications and a combined settings/data/messages workspace. The [component matrix](docs/components/matrix.json) connects each name to its owning issue, specification and reference. The owner accepted the earlier MVP 0.1.0 for local distribution with [recorded limits](docs/verification/xer-120.md); the new `1.0.0-rc.1` candidate has its own [verification record](docs/verification/m3-m4.md).

[Platform adapters](docs/adapters.md) generate WezTerm, Zed, VS Code, Neovim and a bounded KDE colour proof of concept. Work and Gaming are optional accent identities across the same components and token names. The [support matrix](docs/support-matrix.md) distinguishes native results from generation checks, and the [distribution guide](docs/distribution.md) explains local packages, upgrades and remaining acceptance checks.

## Get started

From the project directory, run the following. The setup task installs the locked JavaScript dependencies; obtain permission before installing a missing runtime or library.

```sh
mise run setup
mise run check
mise run build
mise run preview
mise run storybook:build
```

Mise selects Node 26.12.1, Bun 1.4.4 and Python 3.15.0. The token core uses only the Python standard library; its minimum Python version remains 3.10. The Svelte reference has a separate JavaScript dependency set. Inspect `mise.toml`, then run `mise trust` once if mise requests it. Follow [the setup and contribution rules](CONTRIBUTING.md) before installing a missing runtime or library.

Dependencies are locked in `bun.lock`. [XER-102 verification](docs/verification/xer-102.md) records the checks, builds and TypeScript 7 source and consumer probes run for that delivery. Storybook 10.6.3 uses a [declaration-only Bun patch](patches/README.md), with regression checks for valid and invalid story args. Token-only checks remain available through `mise run test`, `mise run validate` and `mise run tokens:build`.

## Project layout

```text
tokens/                    DTCG sources and the versioned contract
  base.tokens.json         Primitives, shared semantic roles and component bindings
  modes/                   Complete mode-specific colour and shadow roles
  themes/                  Theme identity overrides
src/fjord_tokens/          Parsing, composition, validation and CLI
reference/svelte/          Typed Svelte source, Vite specimen and Storybook
scripts/                   Packaging, diagnostic and reproducibility checks
patches/                   Version-specific dependency fixes and upstream licences
adapters/                  Native mappings, generators, fixtures and installation guides
tests/                     Token, platform and type-check regression fixtures
docs/                      Token contract, design guidance and platform mappings
  components/              Framework-independent component specifications
examples/                  Valid theme and override files
  code/                    Annotated syntax, diff, diagnostic and terminal examples
assets/brand/              Canonical SVG logos and usage rules
assets/fonts/              Pinned WOFF2 faces, inspection and original OFL licences
assets/icons/              Pinned solid SVG subset and original MIT notice
dist/                      Generated tokens, library and static specimens; ignored by Git
```

Edit `tokens/` sources. `dist/` files are disposable build results.

## Commands

| Command                      | Result                                                                                  |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| `mise run check`             | Token regression tests, mode/platform validation, reference types and formatting        |
| `mise run validate`          | Validate sources without writing output                                                 |
| `mise run build`             | Write tokens, the typed Svelte library, Vite specimen and native adapters to `dist/` |
| `mise run test`              | Run regression tests using temporary fixtures                                           |
| `mise run preview`           | Write the interactive specimen to `dist/code-and-terminal.html`                         |
| `mise run palette`           | Write colour scales and source/HEX contrast evidence to `dist/palette.html` and `.json` |
| `mise run fonts:build`       | Write self-hosted font assets, typography CSS and `dist/typography.html`                |
| `mise run foundations:build` | Write layout CSS, icon assets and `dist/foundations.html`                               |
| `mise run semantics:build`   | Write semantic CSS, role/state evidence and `dist/semantics.html`                       |

`mise run ci` runs the complete verification sequence. `mise run test:clean` installs and builds in a temporary source copy. [The reference workspace](reference/svelte/README.md) lists development-server commands.

`mise run release:prepare` assembles the local MVP candidate ZIP. `mise run test:candidate` also verifies its file checksums and version consistency, extracts it into a temporary directory, installs the locked dependencies and runs CI from the extracted sources. These tasks do not publish or accept a release. Checksums detect archive damage; they do not authenticate its publisher.

Build a variant using an explicit theme and override:

```sh
mise exec -- python3 -m fjord_tokens build --mode night \
  --theme examples/themes/soft-windows.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/night-compact.tokens.json
```

Builds check the source project, apply `base → mode → theme → overrides`, validate the final values and write the output only after success. Aliases remain in the output, with all their targets included. Repeat `--override` in precedence order; the last file wins for paths it supplies.

Open `dist/code-and-terminal.html` in a browser to compare Night, Day and Aurora. Keep the generated `fonts/` directory beside the HTML for offline use. Use the colour toggle to inspect the text labels and signs on diagnostics, Gitstatus and diff lines. The preview command also accepts `--theme` and repeated `--override` files, validates every mode before writing, and copies the font bundle beside the chosen output.

Validation errors go to stderr and return exit code 1. Invalid CLI arguments return 2. Successful checks return 0 and report token, contrast-pair and platform-value counts. Run `mise exec -- python3 -m fjord_tokens --help` for command help.

## Documentation

- [Architecture and delivery boundaries](docs/decisions/0001-platform-boundaries.md), [dependency policy](docs/dependency-policy.md) and [MIT licence](LICENSE).
- [Token format](docs/token-contract.md), [semantic colours and states](docs/semantic-colors.md), [versions](docs/versioning.md) and [platform mappings](docs/platforms.md).
- [Component meanings](docs/components/glossary.md), [66-component delivery inventory](docs/components/inventory.md) and [primary button specification](docs/components/button.md).
- [Visual principles](docs/design-guidelines.md), [layout and icon foundations](docs/foundations.md), [typography](docs/typography.md), [colour scales](docs/color-scales.md) and [SVG brand assets](assets/brand/README.md).
- [M0 verification](docs/verification/xer-101.md).

The [code and terminal specification](docs/code-and-terminal.md) defines portable role bindings, ANSI behaviour and non-colour markers. [Verification results](docs/verification/xer-108.md) record the checked examples and browser behaviour.

The web reference is Svelte + TypeScript with Vite and Storybook. XER-102 provides the workspace, packaging and diagnostic checks under the latest-stable policy. The TypeScript 7 compiler uses the approved Svelte support arrangement. The library includes nine self-hosted font faces, original licences and typography CSS. Validated DTCG, resolved JSON, CSS and WezTerm exports are described in the [token export guide](docs/token-export.md) and [theme guide](docs/theme-guide.md).

WezTerm 20240203-110809-5046fc22 on macOS is the MVP terminal target. Zed 1.25.3 is the first editor target after MVP, followed by VS Code and Neovim. Application support is recorded after target-specific tests.

The GitHub workflow runs mise checks, token builds and the preview build on pushes and pull requests when the repository is hosted on GitHub. Local checks do not prove a remote CI run. Set `Validate Fjord / validate` as a required status check when configuring repository protection.
