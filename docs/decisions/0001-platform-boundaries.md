# ADR 0001: Shared tokens, Svelte reference and platform adapters

Status: superseded for delivery scope by [ADR 0002](0002-shared-design-foundation.md)

Date: 25 October 2026

Decision authority: Fjord project owner. The owner selected Svelte + TypeScript, the current palette, MIT, WezTerm, Zed first and the TypeScript 7 support arrangement in the approved M0 implementation plan.

Tracking: [XER-101](https://linear.app/xerrion/issue/XER-101/afgraens-tokenkerne-referencekomponenter-og-platformadaptere). Requirements: [design brief](https://chatgpt.com/space/page_bf1179e0aa208191bf1b556beecae213), particularly questions 24, 25, 32, 35 and 36.

This historical record preserves the original decision and rationale. Its component and platform delivery requirements no longer define the active foundation scope. Links below point to the source revision that retains those deliverables.

## Decision

Fjord has one token source, framework-independent component specifications, a Svelte web reference and separate platform adapters. The core owns token names, types, aliases, composition and validation. Framework and platform code consume its outputs.

The web reference uses Svelte + TypeScript, Vite and Storybook. Bun handles JavaScript packages and mise provides task entry points. The [dependency policy](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/dependency-policy.md) defines current version evidence, stable-release selection, exact locking and the approved TypeScript 7 bridge.

## Context and alternatives

The repository contains a working Python token core and a standalone code specimen. The source brief requires configurable themes across websites, applications, editors, terminals and desktop environments, with 66 named components. The [delivery inventory](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/components/inventory.md) assigns 23 to MVP and 43 to the complete library.

| Option | Fit | Cost or limit |
| --- | --- | --- |
| Svelte + TypeScript reference | Selected by the owner; supports a component library and interactive catalogue | Requires Svelte-specific implementation and verified tooling |
| React reference | Could consume the same token exports | Would create a different framework commitment from the selected reference |
| Standalone HTML specimens | Useful for inspecting tokens and examples | Do not provide the reusable component library required by MVP |

Portability comes from shared roles, specifications and explicit mappings. Each target still needs its own implementation and tests.

## Ownership and interfaces

| Part | Owns | Consumes and delivers |
| --- | --- | --- |
| Token core | DTCG sources, semantic roles, aliases, mode completeness, layer order, contrast and syntax separation | Validated mode documents; DTCG currently builds, CSS and resolved JSON are deliverables of XER-116 |
| Component specifications | Anatomy, states, behaviour, accessibility, semantic bindings and terminology | Framework-independent contracts and acceptance examples |
| Svelte reference | Web markup, interaction, typed component interfaces and CSS bindings | Generated token output; reusable Svelte components with types |
| Storybook catalogue | Inspection, examples, theme controls and token explanations | The same Svelte components and token output used by consumers |
| Platform adapters | Target names, version constraints, mapping, capabilities and installation | Resolved semantic values and code/terminal roles; target configuration plus unsupported-role information |

The token core has no Svelte or platform-runtime dependency. Component specifications bind semantic roles. Adapters translate those roles to target properties and report unsupported capabilities. Application services own uploads, persisted answers, message transport and process execution; the glossary specifies the UI boundary.

Implementation homes follow these responsibilities: `src/fjord_tokens/` for the core, `docs/components/` for specifications, `reference/svelte/` for the Svelte library and its Storybook catalogue, and `adapters/<platform>/` for target mappings. XER-102 creates the reference workspace; XER-133 creates the adapter starter. No empty implementation directories are needed to record these boundaries.

The token format, merge order, alias rules and failure behaviour remain defined by the [token contract](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/token-contract.md). An adapter must identify its target/version, supported modes, mapped roles and limits. Missing required roles fail before output; a target-specific substitution must be explicit and documented.

## Design contract

Nordlys is the visual direction. Night uses frost-blue interaction, Day uses deep blue and Aurora uses teal. Rose is a secondary accent. Large surfaces remain cool, opaque and calm, with square window structure and rounded controls.

Noto Sans serves UI text, Raleway serves display/headings and Cascadia Code serves code. Open Sans and Fira Code provide the relevant named fallbacks. XER-105 supplies the [verified self-hosted files, original font licences and typography contract](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/typography.md). Code ligatures, italic comments/keywords, readable comments and the café-toned ANSI yellow follow [design guidance](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/design-guidelines.md) and [code roles](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/docs/code-and-terminal.md).

The supplied SVG logo and approved Day variants are the brand sources. [Brand usage](https://github.com/Xerrion/fjord-design/blob/dec618b921e37723045c29b10cb3d62752bc8fdb/assets/brand/README.md) defines variant selection, size, clear space and colour bindings.

## Platform and accessibility targets

| Target | M0 decision | Verification owner |
| --- | --- | --- |
| Web reference | Latest two stable major releases of Chrome, Edge, Firefox and Safari at release verification | XER-119 records exact browser/OS versions and results |
| MVP terminal | WezTerm 20240203-110809-5046fc22 on macOS | XER-116 records macOS version, mapping, ANSI, cursor and selection results |
| First editor port | Zed 1.25.3 | [XER-140](https://linear.app/xerrion/issue/XER-140/byg-zed-temaer-fra-fjords-faelles-tokenkilde) |
| Following ports | VS Code, then Neovim, then one desktop investigation | XER-134, XER-135 and XER-137 |

These are targets. Support is recorded only after successful application tests. The browser specimen does not establish native terminal or editor support. Colour configuration alone cannot reproduce component layout, window geometry, interactions or every font feature.

Web acceptance targets WCAG 2.2 AA where applicable, native HTML semantics and relevant [APG patterns](https://www.w3.org/WAI/ARIA/apg/patterns/). Verify keyboard operation, focus, screen-reader output, status signals beyond colour and reduced motion. Automated checks supplement that work. See [WCAG 2.2](https://www.w3.org/TR/WCAG22/).

## Licence and distribution

Fjord's own source code, tokens and documentation use the root MIT licence, copyright 2026 Fjord contributors. Third-party packages and bundled assets retain their licences and required notices. Referenced fonts and icons are not evidence that their files are bundled.

MVP distribution consists of local versioned token artifacts, a Svelte library with TypeScript declarations, and a static Storybook build. Generated outputs belong in `dist/`. Source, manifests and lockfiles belong in version control. Package and application-store publication is a release action.

## Delivery and validation

M0 records the decisions and boundaries. M1 establishes the reproducible setup and design foundation; M2 delivers the 23-component reference, exports, catalogue and acceptance. M3 completes the remaining 43 components. M4 starts with the adapter foundation and Zed, followed by VS Code, Neovim and a bounded desktop investigation.

XER-102 must verify the latest dependency set, TypeScript 7 checking in plain TypeScript, invalid Svelte props, and a separate library consumer. XER-116 verifies exported formats and the native WezTerm example. XER-119 records browser and accessibility acceptance. A working catalogue cannot substitute for testing the distributed library.

Revisit this ADR if a selected stable dependency set cannot pass those checks, the owner changes the reference framework, or a target limitation requires changing the shared contract.
