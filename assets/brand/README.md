# Fjord brand assets

Use the SVG sources in this directory. The supplied mark shows steep blue cliff faces around an open fjord, with a muted rose plane at the horizon. The complete logo includes the outlined Fjord wordmark; no font installation is needed to render it.

| Asset | Use |
| --- | --- |
| [fjord-logo.svg](fjord-logo.svg) | Transparent full logo on Night and Aurora surfaces |
| [fjord-mark.svg](fjord-mark.svg) | Transparent mark on Night and Aurora surfaces |
| [fjord-logo-on-dark.svg](fjord-logo-on-dark.svg) | Full logo with its fixed Night background, for a standalone plate |
| [fjord-logo-day.svg](fjord-logo-day.svg) | Transparent full logo on Day surfaces |
| [fjord-mark-day.svg](fjord-mark-day.svg) | Transparent mark on Day surfaces |

The supplied Night files are preserved exactly. Day variants preserve their paths, proportions, viewBox and accessibility attributes; only fill colours differ. Aurora uses the Night brand palette even though its interactive UI accent is teal.

## Day colours

| Part | Semantic binding | Fill |
| --- | --- | --- |
| Wordmark | `semantic.color.ink` | `#16232f` |
| Cliff faces | `semantic.color.frost` | `#215e92` |
| Rose plane | `semantic.color.rose` | `#9b4251` |

These values match the default Day token source. Recheck the bindings when changing the palette. Custom themes need a verified light or dark background for the chosen brand asset; they do not automatically recolour the logo.

## Size, clear space and accessibility

Keep the detailed mark at least 32 CSS pixels wide. When using the full logo, keep its mark portion at least that size and check the wordmark in context. A separate small-size asset is required for uses below 32 pixels; none is supplied here.

Leave clear space of at least one quarter of the displayed mark height on every side. Apply that space in layout rather than changing the SVG viewBox. Scale proportionally. Use the approved fills and supplied wordmark paths; avoid rotation, stretching, added effects or opacity reduction.

Use an appropriate accessible name when the logo identifies Fjord, for example `alt="Fjord"` on an HTML image. Use empty alternative text when the mark only decorates an adjacent visible name. Size must not change the surrounding control's hit area or accessible label.

At 24 pixels, the narrow cliff channels become less distinct. The 32-pixel minimum follows the visual size review on Night and Day. Brand readability and contrast are checked in context; this review does not establish usability at favicon sizes.
