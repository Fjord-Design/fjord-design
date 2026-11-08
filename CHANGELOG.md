# Changelog

## 1.0.0-rc.1 · local candidate · 8 November 2026

- Added the remaining 43 components, exact 66-component matrix, family contracts and a combined settings/data/messages workspace.
- Added Work/Gaming overrides and separately named adapter bundles without changing token names or existing component bindings.
- Added the Neovim generator (native runtime not run), daily-use WezTerm configuration and optional prompt, and an isolated KDE/Qt colour proof of concept.
- Added reproducible full-library local packaging, support/version/migration guides, native check commands and screenshot comparison tooling.
- Fixed stale combobox submission, select constraint validation, hover-card focus/tap handling, disabled-date keyboard entry and bounded numeric inputs during implementation review.
- Retained explicit acceptance limits: unobserved spoken screen-reader output, touch checks unavailable in this browser, Neovim runtime declined, and new editor variants not natively verified. See [verification](docs/verification/m3-m4.md).

## Unreleased

- Added generated Night, Day and Aurora VS Code themes and a reproducible local VSIX. See the [VS Code guide](adapters/vscode/README.md) and [verification record](docs/verification/xer-134.md).
- Rebuilt the declarative adapter foundation and Zed themes from the shared token source. See the [adapter guide](docs/adapters.md) and [Zed verification](docs/verification/xer-140.md).

## 0.1.0 · 29 October 2026

- Added Night, Day and Aurora token sources, scoped Fern accent example, DTCG, JSON, CSS and WezTerm exports.
- Added self-hosted typography, foundations, the 23 MVP components, a Svelte reference and a static Storybook catalogue.
- Added token validation, contrast reports, reproducible builds and a guide for creating a colour identity.
- Accepted by the owner for local distribution with recorded browser, screen reader, terminal and environmental verification limits. See [release evidence](docs/verification/xer-120.md). The candidate-building tasks continue to produce unaccepted archives; running them does not accept or publish a new release.

Fjord sources use MIT. Bundled fonts and icons retain their original licences. The Svelte package version and token contract version are independent of the MVP candidate version.
