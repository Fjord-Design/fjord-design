# Font assets

These nine WOFF2 files are Fjord's self-hosted fonts. Original OFL-1.1 notices stay inside each family directory and accompany every distribution. The repository MIT licence applies to Fjord's own work; it does not replace these font licences.

| Family | Retrieved source | Files | Weight range |
| --- | --- | --- | --- |
| Noto Sans | Google Fonts CSS2, asset revision v42 | Latin normal and italic | 100 to 900 |
| Raleway | Google Fonts CSS2, asset revision v37 | Latin normal and italic | 100 to 900 |
| Open Sans | Google Fonts CSS2, asset revision v44 | Latin normal and italic | 300 to 800 |
| Cascadia Code | Microsoft release v2407.24 | Normal and italic | 200 to 700 |
| Fira Code | Release 6.2 | Normal | 300 to 700 |

Retrieved on 26 October 2026. The Cascadia and Fira versions were the latest stable releases returned by their official GitHub release APIs on that date. Google Fonts asset revisions describe the retrieved CSS responses, not a common semantic release version. Exact URLs, archive entries, byte sizes and SHA-256 hashes are in [manifest.json](manifest.json).

The Google Fonts files are the provider's Latin subsets, downloaded without modification. Danish letters are covered; other writing systems need suitable fallback fonts or separately reviewed subsets. The original source files remain unchanged. All five licence notices are pinned by hash.

## Inspection and updates

1. Retrieve files from the recorded official sources. For Google Fonts, use the [CSS2 API](https://developers.google.com/fonts/docs/css2) for normal/italic variable ranges and retain its matching Latin `unicode-range`. For Cascadia and Fira, select the release archive entries in the manifest.
2. Keep each original licence and update the manifest's source, style, weight, byte size and SHA-256 fields. Review reserved font names before making any font modification.
3. Run `mise exec -- python3 scripts/inspect_fonts.py` with an existing Fontconfig installation. It uses `fc-scan` and the Fontconfig library to inspect the files, compares families/styles/ranges, tests 103 glyphs and writes [inspection.json](inspection.json). Ask before installing missing tools. Native inspection is an asset-update step; ordinary CI checks the exact inspected bytes against this evidence.
4. Run `mise run ci`. Repeat shaping, Danish, fallback and enlargement checks in a consuming implementation before accepting new files.

The probe includes printable ASCII, `æøåÆØÅéÉ`, digits, punctuation and `Il1 O0o`. Fontconfig may report internal families such as Raleway Thin or Fira Code Light for a variable file. CSS assigns the approved family name and the recorded variable weight range.

See the [design guidelines](../../docs/design-guidelines.md#typography) for typography rules and the [consumer guide](../../docs/consumer-guide.md#reuse-assets) for asset use. Standard slanted italic is the default. The [Cascadia documentation](https://github.com/microsoft/cascadia-code) defines its optional cursive `ss01`; [Fira Code](https://github.com/tonsky/FiraCode) documents its own features. Fira's release supplies upright fallback only. Consumers must scope font-specific features so Cascadia's cursive preference does not enable a different Fira feature.
