# Phosphor Fill icons

These 16 SVGs are Fjord's semantic subset of Phosphor Fill. Choose a filename for its meaning and copy the required files into the consuming implementation, retaining the manifest and original licence. Copying requires no build, runtime library or icon font.

Available filenames:

- `check.svg`, `close.svg`, `info.svg`, `success.svg`, `warning.svg`, `danger.svg`
- `add.svg`, `remove.svg`, `search.svg`, `expand.svg`, `next.svg`, `arrow-right.svg`
- `settings.svg`, `file.svg`, `copy.svg`, `loading.svg`

The files retain their source paths, 256-unit viewBox and `currentColor` fill. For inline SVG, `currentColor` can follow the surrounding text colour. Keep the source geometry and fill unchanged; implementations own any generated wrappers.

Use a 24 CSS pixel reference box. Suggested render sizes are 16px for controls, 18px for status and navigation, 20px for larger inline icons, and 24 to 28px for standalone or empty-state icons. Icon size is separate from the hit area. Check the icon in the consuming implementation at its supported sizes: it should be recognizable in context, with an appropriate hit area for any control.

Hide decorative icons from assistive technology and give meaningful standalone icons accessible names. Use visible text for status. `loading.svg` is static; if a consumer makes it a spinner, stop the animation under `prefers-reduced-motion` and retain visible status text. In forced colours, preserve boundaries and focus with system colours rather than relying on shadows or colour alone. See the [design guidance](../../docs/design-guidelines.md#iconography) for sizes and accessible names.

## Provenance and updates

The original SVG assets come from `@phosphor-icons/core` 2.1.1, the latest stable npm release checked on 26 October 2026. Semantic filenames map to upstream names in [manifest.json](manifest.json). The [upstream repository](https://github.com/phosphor-icons/core) documents its SVG layout. The npm version can be newer than the latest GitHub release entry; use the exact registry version and archive integrity recorded in the manifest.

The [original MIT licence](LICENSE) belongs to Phosphor Icons and accompanies every distribution. The manifest records the official npm archive URL, SHA-512 integrity and each file's SHA-256. Archive integrity was checked before reading the selected assets.

1. Select a stable official release and verify the archive against its registry integrity.
2. Retain its licence and selected filled SVGs; update manifest provenance and hashes.
3. From the fjord-design repository root, with mise available, run `mise run ci` to validate the source assets and manifests. It runs checks, including tests and validation, then builds three formats for Night, Day and Aurora.
4. Inspect the icons in a consuming implementation at its supported sizes and system preferences.

The validator permits static filled paths only.
