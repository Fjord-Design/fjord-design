# Typography, Code Block and Kbd

Import `@fjord/svelte/typography.css`, `foundations.css`, `semantics.css` and `type-components.css`. Components/Typography Code Block Kbd demonstrates the components in each mode and the Fern identity.

## Typography

`Typography` takes an `as` element and a visual `role`. Choose heading levels according to document structure. The role selects one of the shared `.fjord-type-*` classes; it does not change the heading level. Body, lead, label, caption, overline and title roles follow [the type scale](../typography.md). Typography content uses the surrounding semantic text colour.

## Code Block

`CodeBlock` accepts a label, the original `code` string and optional highlighted `segments`. Each segment has text and a syntax role. Segments must concatenate to exactly the source or rendering fails. This keeps visual text and clipboard text consistent without accepting HTML. The application supplies highlighting; this component does not parse a language. Use `text` for unclassified source. Empty source is permitted.

The code region preserves whitespace and scrolls horizontally with a keyboard focus target. The surface uses `bg-base`, the background validated for syntax colour contrast. Syntax roles use `syn-*`; comments and keywords use the Cascadia italic face with standard slanted forms. Ligatures follow the document preference and do not alter copied characters. Handwritten forms require the explicit preference described in [typography](../typography.md).

The Copy button writes the original string through the browser clipboard API. The persistent status region reports success or failure. `copyLabel`, `copiedMessage` and `copyFailedMessage` can be localized. Clipboard permission or an insecure context may prevent copying; select and copy the text manually when it fails. The application should avoid placing secrets in source code presented for copying.

## Kbd

`Kbd` renders native `kbd` markup with the code type role and semantic surface, border and text. The application supplies platform-specific labels. For example, use `⌘` on macOS and `Ctrl` on Windows and Linux. Put separate keys in separate elements and include a visible separator for chords.

## Verification

`mise run test:type-components` checks native markup, a named code region, status semantics, escaped source and segment integrity. The packaged consumer checks exported declarations and styles. Browser verification is needed for clipboard permission and feedback, long lines, zoom, ligatures and actual assistive-technology announcements.
