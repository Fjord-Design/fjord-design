# Local distribution and maintenance

The agreed distribution model is a versioned local archive. The owner accepted the complete-library `1.0.0` release on 8 November 2026 with explicit [verification limits](verification/m3-m4.md). The earlier accepted MVP remains historical. No npm registry, editor marketplace or desktop store publication is part of this release. Generated output stays out of Git.

From the repository root:

```sh
mise run ci
mise run release:library
mise run test:library
```

`release:library` builds tokens, Svelte, Storybook, all five platform bundles, Work/Gaming and contrast evidence. It assembles `dist/release/fjord-library-1.0.0.zip` twice and requires byte equality. The adjacent `.zip.sha256` detects transfer damage. Archive entries have stable order, timestamps and permissions. The manifest records every member's SHA-256, component count, independent token/Svelte versions and support-matrix path. Checksums do not authenticate the publisher.

The manifest marks this release `accepted-local` with distribution `local`. Its acceptance entry names Xerrion, the date `2026-11-08`, and `docs/verification/m3-m4.md` as the decision record. These fields record owner acceptance. They do not prove a registry publication or native compatibility beyond the support matrix.

The ZIP contains `svelte/`, `storybook/`, token exports, source project and documentation. Generated native packages are under `platforms/`, with Work/Gaming under `variants/`. Adapter source remains under `adapters/`; these paths cannot overwrite each other. VS Code includes a base VSIX and separately named variant VSIX files. Neovim includes generation-only Lua schemes and fixtures. KDE includes the bounded `.colors` proof of concept. Every target includes installation, update and removal instructions and its licence/report.

The archive retains original font/icon licences, bundled upstream notices and recursive installed-dependency notices, including nested vendor notices. See `licenses/`, `assets/` and `third-party-notices/index.json` in the package. Retain them when redistributing the corresponding assets. The theme generators introduce no runtime dependency into the token core.

`test:library` verifies safe paths, exact archive membership, checksums and matching version declarations before extraction. It installs the already locked dependency set in a temporary directory and runs full CI from the extracted sources, without reusing prebuilt outputs. Permission is still required before installing a missing runtime or library. The historical `release:prepare`/`test:candidate` tasks retain their MVP archive name; use the library tasks for this milestone.

To consume Svelte, use `mise run test:types` to produce `dist/fjord-svelte-1.0.0.tgz`. Install that local package with the consuming project's package manager. Alternatively, copy `svelte/` from the complete archive into a local package source. Keep its fonts, icons, CSS and licences together. The static catalogue can be served from `storybook/` for review. Follow the relevant native README instead of copying a whole application configuration.

## Compatibility and release ownership

Xerrion owns release acceptance and adapter maintenance. Svelte follows SemVer independently from token contract `0.3.0` and editor-theme package `0.1.0`. On 8 November 2026, the owner accepted `1.0.0` with the documented limits, including the follow-up Slider and selection checks. This satisfies the candidate's explicit acceptance requirement. Acceptance does not convert untested targets into supported targets. Future releases still require owner acceptance after applicable checks and review of remaining limits.

After 1.0, removing or renaming exports, changing required props or altering established interaction contracts requires a major version. Additive exports and optional props use minor versions. Compatible fixes use patches. Target-version support is declared only after testing that target.

For deprecation, name the replacement, publish an example and changelog entry, retain the old binding for the supported major and remove it only in a new major. Token deprecation follows the separate [token lifecycle](versioning.md). No token paths or MVP exports are deprecated by this release.

For each release: update source/lock metadata, changelog, compatibility/migration notes and target matrix together; run CI, extracted-archive rebuild and applicable native/browser checks; inspect screenshot differences; record skipped checks without converting them into passes. Retain the previous archive/configuration for rollback. Do not change a reviewed baseline solely to make a check pass. External publication or additional targets need an explicit distribution decision.
