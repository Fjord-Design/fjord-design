# ADR 0002: Keep only the shared design foundation

Status: accepted

Date: 8 November 2026

Authority: the project owner asked for a shared design foundation that separate implementations can inherit, then explicitly asked to cut the project to its essentials.

Supersedes the delivery scope of [ADR 0001](0001-platform-boundaries.md).

## Decision

Keep design tokens and rules, inheritance, fonts, icons, brand assets, portable exports and necessary validation. The Python CLI provides `validate`, `build` and `export`; default tasks require no application runtime or third-party Python library.

Component libraries, platform adapters, visual catalogues, previews and release-packaging infrastructure are outside this repository's active scope. Separate projects consume the foundation through its exported data and source assets. Existing implementations and their evidence remain in [Git history](https://github.com/Xerrion/fjord-design/tree/dec618b921e37723045c29b10cb3d62752bc8fdb).

## Reason and trade-off

The combined project made a reusable design foundation depend on a component roadmap, framework tooling and several native application packages. Keeping those implementations in optional folders would still leave their maintenance in this project. Removing them establishes the requested boundary with fewer tasks and dependencies.

The cost is that consumers own their mapping, installation, runtime behaviour and verification. The foundation retains semantic roles and composition rules without promising equivalent native rendering.

## Compatibility and verification

Token names, values, schema and layer order remain unchanged. Removing CLI commands and implementation packages is an explicit unreleased breaking delivery change; the historical combined release and its acceptance remain unchanged.

Foundation checks must validate the retained token contract and assets, and build DTCG, resolved JSON and CSS for all three modes. They must not require implementation code, JavaScript dependencies, platform documentation or native applications. Consumers verify their own target behaviour.

Revisit this decision if a proposed feature cannot be expressed as shared design data or validation without introducing a target-specific dependency.

## Test-tooling update: 11 November 2026

The owner approved native pytest for regression tests, with development-only dependencies locked by uv and run through mise. This updates the original test-tooling restriction, not the runtime boundary: the CLI, validation and exports still use only Python's standard library and require no application runtime.

