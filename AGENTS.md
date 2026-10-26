# Agent guidance

Fjord is a design system with shared DTCG tokens, a Python token core, framework-independent component specifications, and a Svelte and TypeScript web reference. Platform adapters consume the shared contract and need their own verification.

## Where to work

- Edit token sources in `tokens/` and token processing in `src/fjord_tokens/`. Use `tests/` for regression coverage.
- Keep component contracts in `docs/components/` and the Svelte implementation and stories in `reference/svelte/`.
- Treat `dist/` as generated output. Change its sources and regenerate it; do not commit generated output, `node_modules/`, or Svelte build caches.
- Keep third-party asset licences and notices with their assets. When changing the icon subset, follow `assets/icons/README.md` and regenerate the typed icon data.

## Contracts

- For token changes, follow `docs/token-contract.md`. Each mode must be complete, component tokens must alias semantic roles, and changes must preserve the contract's validation and contrast checks.
- For component or visual changes, use the relevant specification in `docs/components/` and `docs/design-guidelines.md`. Preserve native semantics, keyboard access, visible focus, reduced-motion support, and status cues beyond colour where applicable.
- Keep the token core independent of Svelte and platform runtimes. See `docs/decisions/0001-platform-boundaries.md` for ownership and target-specific verification. Do not claim platform support without testing the target.
- Follow `docs/dependency-policy.md` for dependency changes and `patches/README.md` for the version-specific Storybook patch. Update the manifest, lockfile, patches, and notices together when applicable. Do not edit cached dependencies.

## Checks

Run commands from the repository root. `mise.toml` defines tool versions and task dependencies.

- Run `mise run test` and `mise run validate` for focused token checks.
- Run `mise run check` for the combined local checks. Some tasks generate files in `dist/`.
- Run `mise run ci` before committing. It includes builds, type checks, Storybook, and artifact checks. Use `mise run test:clean` when a clean-install check is needed; it installs dependencies in a temporary copy.
- Obtain permission before installing a missing runtime or library. `mise run setup` installs the locked JavaScript dependencies.

Choose additional checks for the affected behavior, including browser and accessibility checks for interaction changes. Report what ran, what failed or was skipped, and any remaining verification limits. A local check does not establish remote CI or platform support.

## Working practices

Inspect the relevant code and documentation before changing behavior. Preserve unrelated work, including untracked files. Work on a feature branch and use focused conventional commits when committing. Never commit with failing required checks; report a blocker if a check cannot run.
