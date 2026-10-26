# Contributing to Fjord

Use a feature branch and keep each conventional commit focused on one change. Run `mise run ci` before committing. A failed check must be fixed before the change is committed.

## Setup

Read `mise.toml` before trusting it. Mise selects Node 26.12.1, Bun 1.4.4 and Python 3.15.0. Obtain permission before installing a missing runtime or library. After approval, install missing runtimes with `mise install`, then run `mise run setup` to install the exact dependency set from `bun.lock`. Normal builds reuse that set.

Work from the repository root:

```sh
mise run check
mise run ci
mise run test:clean
```

`check` runs the token tests, source validation, reference type check and reference formatting check. `ci` also builds the artifacts and Storybook, verifies deliberate type failures in source/consumers and Storybook args, and checks deterministic token/specimen output. `test:clean` repeats that sequence in a temporary copy containing only files eligible for version control. It does not create a commit or contact GitHub.

Run `mise run format` to fix formatting in the frontend workspace, type fixtures, JavaScript scripts and CI configuration. Existing token-core Python style is preserved. Keep Python changes readable and run its regression tests.

## Ownership

| Location | Edit here for |
| --- | --- |
| `tokens/`, `src/fjord_tokens/`, `tests/` | Token contract, composition, validation and regression tests |
| `docs/components/` | Framework-independent component contracts |
| `reference/svelte/` | Svelte source, typed interfaces, stories and frontend configuration |
| `patches/`, `tests/types/` | Dependency fixes, upstream notices and type-safety regressions |
| `adapters/<platform>/` | Future platform implementations owned by their delivery tasks |
| `assets/brand/` | Approved SVG sources and their usage rules |
| `assets/fonts/`, `assets/icons/` | Pinned third-party assets, provenance and original licences |

`dist/`, `node_modules/`, `.svelte-kit/` and `.svelte-check/` are generated. Do not commit them. Commit source manifests and `bun.lock`. The adapter starter belongs to XER-133; do not add empty directories to imply an implementation exists.

Dependency changes follow [the dependency policy](docs/dependency-policy.md). Check current stable releases and peer/engine constraints, update exact manifest versions and regenerate the lockfile together. Keep the approved TypeScript support arrangement until the same diagnostic checks pass without it.

Version-specific dependency patches are declared in the root manifest and applied by Bun on frozen installs. Review [their scope and removal conditions](patches/README.md) when upgrading. Commit the patch, notices, manifest and lockfile together. Do not edit cached dependencies directly.

Fjord sources use [MIT](LICENSE). Retain dependency licences and notices. Font files and third-party artwork require their own licence verification before they are bundled.
