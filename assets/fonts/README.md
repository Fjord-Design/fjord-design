# Font assets

These nine WOFF2 files are Fjord's self-hosted fonts. Copy the files needed by the consuming implementation, retaining the manifest and original licence notices. Copying them needs no foundation tooling or build; the foundation does not install fonts for consumers.

## Choose and use fonts

| Family and role | Files | Weight range |
| --- | --- | --- |
| Noto Sans, UI and reading text | `noto-sans/NotoSans-latin.woff2`, `noto-sans/NotoSansItalic-latin.woff2` | 100 to 900 |
| Raleway, display, headings and dialog titles | `raleway/Raleway-latin.woff2`, `raleway/RalewayItalic-latin.woff2` | 100 to 900 |
| Open Sans, fallback for Noto Sans | `open-sans/OpenSans-latin.woff2`, `open-sans/OpenSansItalic-latin.woff2` | 300 to 800 |
| Cascadia Code, code | `cascadia-code/CascadiaCode.woff2`, `cascadia-code/CascadiaCodeItalic.woff2` | 200 to 700 |
| Fira Code, fallback for Cascadia Code | `fira-code/FiraCode-VF.woff2` | 300 to 700 |

Configure the chosen files and readable fallbacks in the consuming implementation, or use documented fallbacks instead of bundling WOFF2. Assign the approved family names and recorded variable weight ranges in CSS. Raleway uses weight 600 for its roles and 700 for title-4. The normal and italic files provide the corresponding styles; Fira's release supplies upright fallback only.

Check the result in the consumer: text should remain readable with the intended font or its fallback, including Danish text and enlarged text. Keep long labels and paths wrapping. The Google Fonts files are the provider's Latin subsets, downloaded without modification. Danish letters are covered; other writing systems need suitable fallback fonts or separately reviewed subsets.

Cascadia uses standard slanted italic by default. Ligatures default on and optional handwritten cursive defaults off; users can select them independently. The [Cascadia documentation](https://github.com/microsoft/cascadia-code) defines its optional cursive `ss01`; [Fira Code](https://github.com/tonsky/FiraCode) documents its own features. Scope font-specific features so Cascadia's cursive preference does not enable a different Fira feature.

See the [design guidelines](../../docs/design-guidelines.md#typography) for typography rules and the [consumer guide](../../docs/consumer-guide.md#reuse-assets) for asset use.

## Sources and licences

| Family | Retrieved source |
| --- | --- |
| Noto Sans | Google Fonts CSS2, asset revision v42 |
| Raleway | Google Fonts CSS2, asset revision v37 |
| Open Sans | Google Fonts CSS2, asset revision v44 |
| Cascadia Code | Microsoft release v2407.24 |
| Fira Code | Release 6.2 |

The files were retrieved on 26 October 2026. Cascadia and Fira were the latest stable releases returned by their official GitHub release APIs on that date. Google Fonts asset revisions describe the retrieved CSS responses, not a common semantic release version. The original source files remain unchanged. Exact URLs, archive entries, byte sizes and SHA-256 hashes are in [manifest.json](manifest.json), which also records `fontDisplay` as `swap`.

All five families use OFL-1.1. Their original notices remain in each family directory, are pinned by hash and must accompany every distribution. Consult those notices for the terms. The repository MIT licence applies to Fjord's own work; it does not replace the font licences.

## Inspection and updates

1. Retrieve files from the recorded official sources. For Google Fonts, use the [CSS2 API](https://developers.google.com/fonts/docs/css2) for normal and italic variable ranges and retain its matching Latin `unicode-range`. For Cascadia and Fira, select the release archive entries in the manifest.
2. Keep each original licence and update the manifest's source, style, weight, byte size and SHA-256 fields. Review reserved font names before making any font modification.
3. From the fjord-design repository root, with mise (Python 3.15.0 via `mise exec python@3.15.0`) and an existing Fontconfig installation available, run:

   ```sh
   mise exec python@3.15.0 -- python3 scripts/inspect_fonts.py
   ```

   Alternatively, with the pinned Python and Fontconfig already available, run `python3 scripts/inspect_fonts.py` from the same directory. The script uses `fc-scan` and the Fontconfig library to compare families, styles and variable weight ranges, test 103 glyphs and write [inspection.json](inspection.json). Ask before installing missing tools. A successful inspection of the supplied set prints `PASS inspected 9 WOFF2 files: family, style, variable weights and 103 glyphs`.
4. From the repository root, run `mise run ci`. It runs checks, including tests and validation, then builds three formats for Night, Day and Aurora. Repeat shaping, Danish, fallback and enlargement checks in a consuming implementation before accepting new files.

The probe covers printable ASCII, `æøåÆØÅéÉ`, digits, punctuation and `Il1 O0o`. Fontconfig may report internal families such as Raleway Thin or Fira Code Light for a variable file. Native inspection is an asset-update step; ordinary CI checks the exact inspected bytes against this evidence. The inspection does not establish consumer shaping or rendering.
