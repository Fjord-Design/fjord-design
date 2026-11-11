# Fjord brand assets

These SVGs contain the Fjord mark and full logo. The mark shows steep blue cliff faces around an open fjord, with a muted rose plane at the horizon. The full logo includes the supplied outlined Fjord wordmark, so it does not need a font installation.

## Choose and use an asset

| Asset | Use |
| --- | --- |
| [fjord-logo.svg](fjord-logo.svg) | Transparent full logo on Night and Aurora surfaces |
| [fjord-mark.svg](fjord-mark.svg) | Transparent mark on Night and Aurora surfaces |
| [fjord-logo-on-dark.svg](fjord-logo-on-dark.svg) | Full logo with its fixed Night background, for a standalone plate |
| [fjord-logo-day.svg](fjord-logo-day.svg) | Transparent full logo on Day surfaces |
| [fjord-mark-day.svg](fjord-mark-day.svg) | Transparent mark on Day surfaces |

Night is the default dark, frost-accent mode. Day is light with a deep-blue accent. Aurora is a deeper dark mode with a teal UI accent, but uses the Night brand palette.

Copy the appropriate SVG into the consuming implementation; no foundation tooling or build is needed simply to copy it. Scale it proportionally and check that the mark and, for the full logo, the wordmark are readable against the actual background. Brand readability and contrast must be checked in context.

Keep the detailed mark at least 32 CSS pixels wide. In the full logo, keep the mark portion at least that wide and check the wordmark in context. Uses below 32 pixels require a separate small-size asset; none is supplied here. At 24 pixels, the narrow cliff channels become less distinct. The 32-pixel minimum follows the visual size review on Night and Day, which does not establish usability at favicon sizes.

Leave clear space of at least one quarter of the displayed mark height on every side. Apply it in layout rather than changing the SVG viewBox. Use the approved fills and supplied wordmark paths; avoid rotation, stretching, added effects or opacity reduction.

When the logo identifies Fjord, give it an appropriate accessible name, for example `alt="Fjord"` on an HTML image. Use empty alternative text when the mark only decorates an adjacent visible name. The asset's size must not change the surrounding control's hit area or accessible label.

## Colours and maintenance

The supplied Night files are preserved exactly. Day variants preserve their paths, proportions, viewBox and accessibility attributes; only fill colours differ. The SVG roots have `role="img"` and `aria-label="Fjord"`.

| Day part | Semantic binding | Fill |
| --- | --- | --- |
| Wordmark | `semantic.color.ink` | `#16232f` |
| Cliff faces | `semantic.color.frost` | `#215e92` |
| Rose plane | `semantic.color.rose` | `#9b4251` |

These fixed fills match the default Day token source. Recheck the bindings when changing the palette. Custom themes need a verified light or dark background for the chosen asset; they do not automatically recolour the logo.
