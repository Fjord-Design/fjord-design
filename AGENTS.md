# Agent guidance

Fjord contains the shared design foundation: tokens, inheritance, design rules, fonts, icons, portable exports and validation. Implementations belong in separate projects.

## Scope and ownership

- Edit design values in `tokens/`, processing in `src/fjord/` and regression tests in `tests/`.
- Keep design and consumer guidance in `docs/`. Preserve original asset licences and provenance under `assets/`.
- Keep the CLI limited to `validate`, `build` and `export`; keep default tasks independent of application runtimes.
- Do not add component libraries, platform adapters, previews, catalogues or release infrastructure without a new scope decision.
- Treat `dist/` as generated output. Change sources and regenerate; do not commit generated files or caches.

## Contracts

Follow `docs/token-contract.md` and `docs/design-guidelines.md`. Each mode must be complete, component tokens must alias semantic roles, and overrides must preserve the declared types and validation rules. Keep source values and their meaning authoritative; implementations own target mappings and runtime behaviour.

Use the Python standard library for runtime code. Native pytest and its dependencies are development-only: declare them in `pyproject.toml`, lock them in `uv.lock` and run them through the mise-pinned Python and uv tools. `mise run test` synchronizes the ignored `.venv` and runs pytest; the equivalent direct runner is `mise exec -- uv run --locked --python python3 -m pytest`. Keep mise's absolute `PYTHONPATH` for CLI subprocess tests and its uv interpreter preference. `mise exec -- uv sync --locked --python python3` provisions the environment without running tests. Obtain permission before installing a missing runtime or tool, or adding a dependency. Font maintenance may use the existing native inspection probe in `scripts/inspect_fonts.py`; ordinary checks must remain independent of native font tools.

## Verification and Git

Run `mise run test` and `mise run validate` for focused checks. Run `mise run ci` before committing; it checks and builds the foundation. Run relevant regression coverage after changing composition, exports or asset validation.

Inspect the affected sources before changing them. Preserve unrelated work, including untracked files. Use a feature branch and focused conventional commits. Never commit with failing required checks. Report executed checks and remaining limits; foundation validation does not establish platform support.

The current boundary is recorded in `docs/decisions/0002-shared-design-foundation.md`.
