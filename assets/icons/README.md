# Phosphor Fill icons

These 16 original SVG assets come from `@phosphor-icons/core` 2.1.1, the latest stable npm release checked on 26 October 2026. Paths, the 256-unit viewBox and `currentColor` fill remain unchanged. Semantic filenames map to upstream names in [manifest.json](manifest.json).

The [original MIT licence](LICENSE) belongs to Phosphor Icons and accompanies every distribution. The manifest records the official npm archive URL, SHA-512 integrity and each file's SHA-256. Archive integrity was checked before reading the selected assets. No runtime library or icon font is required.

The subset covers check, close, info, success, warning, danger, add, remove, search, expand, next, arrow-right, settings, file, copy and loading. The [upstream repository](https://github.com/phosphor-icons/core) documents its SVG layout. The npm version can be newer than the latest GitHub release entry; use the exact registry version and archive integrity recorded here.

1. Select a stable official release and verify the archive against its registry integrity.
2. Retain its licence and selected filled SVGs; update manifest provenance and hashes.
3. Run `mise run ci` to validate the source assets and manifests.
4. Inspect the icons in a consuming implementation at its supported sizes and system preferences.

The validator permits static filled paths only. Implementations own any generated wrappers. [Design guidance](../../docs/design-guidelines.md#iconography) defines sizes and accessible names.
