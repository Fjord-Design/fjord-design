# Dependency policy

Choose the latest stable release of every direct library and development dependency when setting up or upgrading the reference. Check the registry again at implementation time. Stable means the package's `latest` release, excluding alpha, beta, release-candidate, nightly and canary builds.

Record exact versions in the manifest and commit the Bun lockfile. Reuse that lockfile for reproducible installs and CI; a normal build must not resolve a new dependency set. Resolve transitive dependencies within their publishers' supported ranges. Upgrades update the manifest, lockfile and verification together.

Mise provides the common task entry points. Bun manages JavaScript packages. Python token consumers do not need a JavaScript runtime. Obtain permission before installing missing tools or libraries, as required by the project instructions.

## Registry evidence

Checked against npm again on 26 October 2026. Exact versions are declared in the root and reference workspace manifests. Installation and compatibility evidence are tracked in [XER-102 verification](verification/xer-102.md).

| Package | Stable release | Purpose |
| --- | --- | --- |
| `svelte` | 5.57.2 | Web reference framework |
| `typescript` | 7.0.2 | Native TypeScript compiler |
| `vite` | 8.3.5 | Reference and library build |
| `storybook` | 10.6.3 | Interactive catalogue |
| `@storybook/svelte-vite` | 10.6.3 | Svelte/Vite integration |
| `@storybook/react` | 10.6.3 | Types referenced by Storybook's shared declarations |
| `@types/react` | 19.3.0 | Storybook manager declaration dependencies |
| `@types/react-dom` | 19.3.0 | Storybook renderer declaration dependencies |
| `@sveltejs/vite-plugin-svelte` | 7.3.1 | Svelte build integration |
| `svelte-check` | 4.7.7 | Svelte diagnostics and type checking |
| `@sveltejs/package` | 3.0.1 | Svelte source and declaration packaging |
| `@types/node` | 26.6.5 | Build-configuration types |
| `prettier` | 3.9.10 | Source formatting |
| `prettier-plugin-svelte` | 4.1.1 | Svelte formatting |
| `typescript` support package | 6.0.3 | Approved Svelte compiler-API bridge |

Inspect `npm view <package> version peerDependencies engines --json` when updating. Package metadata and [Storybook's Svelte/Vite documentation](https://storybook.js.org/docs/get-started/frameworks/svelte-vite) establish declared requirements. Build and type-check results establish whether the selected set works in Fjord.

The selected runtimes are Node 26.12.1, Bun 1.4.4 and Python 3.15.0, pinned in mise. Node 26.12.1 satisfies the declared Vite, Svelte plugin, packaging and formatter engine requirements. Node and Bun include the fixes for [CVE-2026-111497](https://nodejs.org/en/blog/vulnerability/october-2026-security-releases) in their bundled Brotli decoders; see also the [Bun advisory](https://github.com/oven-sh/bun/security/advisories/GHSA-663w-cv8g-jj2p). Stories use TypeScript CSF, so a separate Svelte story-format addon is unnecessary.

Installation was approved and completed on 26 October 2026. `mise run setup` installs from `bun.lock` with the frozen-lockfile check. The React-related development packages satisfy references in Storybook's shared type declarations; Fjord components use Svelte. Storybook's own Svelte docgen dependency also resolves its supported TypeScript 5.x range internally. Fjord's checker explicitly invokes TypeScript 7; no override forces tool internals to use a different compiler API.

## TypeScript 7 and the Svelte bridge

The owner approved one explicit exception: the latest TypeScript 6.x release may provide Svelte tooling's compiler API while TypeScript 7 performs type checking. Svelte's [official instructions](https://github.com/sveltejs/language-tools/blob/master/packages/svelte-check/README.md#typescript-7-supports) require both versions.

The documented arrangement uses the package name `typescript` for 6.x and the npm alias `@typescript/native` for TypeScript 7. With the observed releases, the intended entries are:

```json
{
  "typescript": "6.0.3",
  "@typescript/native": "npm:typescript@7.0.2"
}
```

The reference manifest contains this arrangement. M0 did not install frontend dependencies or run compatibility checks. Use the officially documented `svelte-check --tsgo` path and verify which compiler actually runs. Its documented incremental-checking limits require particular attention to imported Svelte files and library consumers.

TypeScript 7.0.2 exposes `bin/tsc`, rather than a `tsgo` executable. The diagnostic probes resolve the compiler through `@typescript/native` and check its reported version. The reference check recreates svelte-check's generated overlays because 4.7.7 otherwise retains deleted virtual Svelte files in this setup.

XER-102 must prove all of the following with the selected releases:

1. A deliberate type error in ordinary TypeScript fails the check.
2. An invalid prop passed to a Svelte component fails the check.
3. A separate consumer of the built library receives useful declarations and rejects invalid use.
4. Valid code passes the same checks, library build and Storybook build.

Do not force incompatible peers, silently downgrade dependencies, suppress diagnostics or replace checking with transpilation. If the selected set cannot meet these requirements, record the exact blocking package and failed check in XER-102. Resolve the conflict before declaring the setup complete.

Remove the support-package exception when official Svelte tooling supports the current compiler without it and the same checks pass.

## Storybook declaration patch

Storybook 10.6.3 is still the latest stable release checked on 26 October 2026. Its shared React factory declarations produced four errors with both TypeScript 7.0.2 and 6.0.3. At the owner's request, a [version-specific Bun patch](../patches/README.md) corrects the generic type relationships in one declaration file. Runtime JavaScript and package versions are unchanged.

The root manifest and lockfile declare the patch, and Bun applies it during frozen installation. The full reference check, library/Storybook builds, source/consumer type probes and five negative Storybook-args cases now pass, including from a clean source copy. No compiler check is disabled and no diagnostic suppression is added. The original third-party MIT notice is retained with the patch.

Remove the patch when a later stable Storybook release passes the same checks without it. Keep any continued patch explicit and version-bound. See [the verification results](verification/xer-102.md).
