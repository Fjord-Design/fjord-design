# Typography

Noto Sans supplies UI and reading text, Raleway supplies headings, and Cascadia Code supplies code. The shared DTCG sources own the scale. All three colour modes use the same typography values.

Run `mise run fonts:build` and open `dist/typography.html`. The specimen includes Danish characters, confusable glyphs, comments, long text and controls for modes, fallback fonts, text enlargement, ligatures and cursive italic.

## Families and assets

| Role | Stack, in order | Bundled faces and variable weights |
| --- | --- | --- |
| UI and reading | Noto Sans, Open Sans, system-ui, -apple-system, Segoe UI, sans-serif | Noto Sans normal/italic 100 to 900; Open Sans normal/italic 300 to 800 |
| Headings | Raleway, Noto Sans, system-ui, sans-serif | Raleway normal/italic 100 to 900 |
| Code | Cascadia Code, Fira Code, JetBrains Mono, ui-monospace, Menlo, Consolas, monospace | Cascadia normal/italic 200 to 700; Fira Code normal 300 to 700 |

JetBrains Mono and the system families are optional installed fallbacks. The five bundled families use original OFL-1.1 licences, kept with their files. The [asset manifest](../assets/fonts/manifest.json) pins source URLs, releases where supplied, byte sizes, hashes, styles and weight ranges. [Asset instructions](../assets/fonts/README.md) describe retrieval and inspection.

Noto Sans, Raleway and Open Sans use the provider's unmodified Latin WOFF2 subsets. They cover Danish and the recorded Latin probe. Other scripts use available fallback fonts; this bundle does not claim the full international Noto character set. Cascadia and Fira retain the files supplied in their release archives. No system installation is needed.

## Type scale

Sizes and tracking below are CSS pixels at a 16px root. The generated stylesheet converts them to `rem`, so the user's root text size scales the type. Line heights are unitless ratios in tokens; the table gives the resulting line height at the default size. The complete numeric values remain in DTCG and `dist/typography.json`.

| Token suffix | Family | Size / line height | Weight | Tracking | Use |
| --- | --- | --- | --- | --- | --- |
| type-display | Raleway | 40 / 48 | 600 | -0.4 | Page display heading |
| type-title-1 | Raleway | 30 / 38 | 600 | 0 | Page title |
| type-title-2 | Raleway | 24 / 32 | 600 | 0 | Section title |
| type-title-3 | Raleway | 18 / 26 | 600 | 0 | Subsection or dialog title |
| type-title-4 | Raleway | 15 / 22 | 700 | 0 | Compact heading |
| type-lead | Noto Sans | 17 / 28 | 400 | 0 | Introductory reading text |
| type-body | Noto Sans | 15 / 24 | 400 | 0 | Body text |
| type-body-strong | Noto Sans | 15 / 24 | 600 | 0 | Body emphasis |
| type-label | Noto Sans | 13 / 16 | 500 | 0 | Control labels |
| type-small | Noto Sans | 13 / 18 | 400 | 0 | Help text |
| type-caption | Noto Sans | 12 / 16 | 400 | 0 | Captions and metadata |
| type-overline | Noto Sans | 11 / 16 | 600 | 0.88 | Short uppercase section label |
| type-code | Cascadia Code | 14 / 22 | 400 | 0 | Code and terminal text |
| type-code-small | Cascadia Code | 12 / 18 | 400 | 0 | Compact code metadata |
| type-code-italic | Cascadia Code | 14 / 22 | 400 | 0 | Comments and keywords |

Each full path starts with `semantic.typography.`. The generated class prefixes the suffix with `fjord-`, for example `.fjord-type-body`. Text remains sentence case except an explicitly uppercase overline. Keep important instructions in body or help text. Small metadata styles are not the default for reading paragraphs.

## Code features

Contextual code ligatures are enabled by default through `calt` and `liga`. Comments and keywords use the real Cascadia italic face with standard slanted forms. Handwritten cursive is off by default. Cascadia's optional cursive forms require its `ss01` styleset. The stylesheet gives that styleset a font-specific name through `@font-feature-values`. Setting `data-fjord-cursive="on"` applies `font-variant-alternates: styleset(fjord-cursive)` to code italic. Fira Code's unrelated `ss01` is not enabled.

Expose these preferences in the consuming application's settings. Set attributes on the document root:

```html
<html data-fjord-ligatures="off">
```

Omit the attributes for the defaults: ligatures on and handwritten forms off. Disabling ligatures retains each source character. Applications can offer ligatures and handwritten forms independently. Set `data-fjord-cursive="on"` only when the user requests handwritten forms; `off` or an absent attribute retains standard italic. Browsers without named styleset support retain standard italic.

Fira Code 6.2 contains no italic face. With `font-synthesis: none`, its fallback remains upright. Comments retain their semantic colour and source comment markers. The specimen's fallback control lets you inspect this degradation. The system-font control omits all named web fonts.

## Self-hosting and distribution

The stylesheet uses relative WOFF2 URLs and `font-display: swap`. Text appears through the fallback stack while fonts load. There is no runtime request to a font CDN and no `local()` lookup that could silently select a different installed version. Serve the stylesheet and its family directories together; permit their origin in your font/content policy.

The local Svelte archive contains the stylesheet, all nine fonts, their manifest, inspection evidence and five original licence files. Import once at the application entry:

```ts
import '@fjord/svelte/typography.css';
```

```html
<h1 class="fjord-type-title-1">Theme settings</h1>
<p class="fjord-type-body">Choose how Fjord looks in your workspace.</p>
<code class="fjord-type-code">const mode = "night";</code>
```

A framework-independent consumer can copy `dist/fonts/` and link `fonts/typography.css`. Keep the directory structure and OFL notices. Fonts retain OFL-1.1; Fjord's own sources retain MIT. Do not rename, modify or subset fonts without reviewing their original licence, including any reserved font names. Generic colour/token CSS and resolved-JSON export remain XER-116 work.

## Readability and checks

Use semantic text colours at full opacity. Comments use `syn-comment`; the [code and terminal contract](code-and-terminal.md) enforces their contrast. Keep long prose and file paths wrapping. Code may use a labelled, keyboard-scrollable region when preserving lines is necessary. Avoid fixed-height text containers and do not disable browser zoom. Users must be able to enlarge text to 200 percent without losing content or controls.

`mise run fonts:validate` checks hashes, original notices, WOFF2 headers, saved face inspection and primary weight/style coverage for all 15 roles in every mode. `mise run check` includes this validation and font regression tests. Package-consumer checks confirm that stylesheet URLs resolve to the exact pinned files with their licences. These repeatable checks use the standard library and do not require native font tools.

[XER-105 verification](verification/xer-105.md) records the browser, enlargement, fallback and shaping results. The cross-browser release matrix remains owned by XER-119.
