# Design guidelines

Fjord is a calm, cool, Nordic design foundation. Separate implementations apply these shared values and rules to their own surfaces. It is a quiet workspace by day and a personal room at night. When rules conflict, **calm wins** - over density, over personality, over novelty.

## Principles

- **Calm first.** Colour is an accent, never a surface. Large areas are always one of the `bg-*` blues; the café and Nordic hues appear in small doses: a button, a badge, a line of code.
- **Cool ground, warm sparks.** The ground is a cool dark blue (`bg-base`, seeded from oklch(20.88% 0.018 245)). Warmth comes from café tones (`latte`, `foam`) and `rose`, kept muted so they glow instead of shout.
- **Discreet, but recognisably yours.** No neon, no synthwave gradients, no baroque ornament, no shabby texture. Personality lives in the combination: frost-blue interaction, café-toned code, square windows with round buttons.
- **Clear contrast.** Every text token meets 4.5:1 on the surfaces its note names, in every mode. Comments in code are content: `syn-comment` holds 7.5:1 on `bg-base` in Night.
- **Quiet depth.** Floating surfaces use `shadow-raised` or `shadow-overlay` with a faint top highlight. Keep panels and windows opaque, without blur. Separate regions with 1px hairlines and fill changes. A 3px `accent` edge marks the current navigation item; do not add it to cards or alerts.

## Modes

| Mode | id | When | Character |
| --- | --- | --- | --- |
| Night | `night` | Default. Home office, evenings, anywhere dim. | Cool dark blue, frost-blue `accent`. |
| Day | `day` | Bright offices and daylight. | Pale blue-grey ground, deep blue `accent`; every hue darkened to stay legible. |
| Aurora | `aurora` | Gaming and late nights. | Deeper, almost-black blue ground; aurora-teal `accent`; the palette a touch brighter. |

Night is the default mode. Each mode supplies complete values for the same token names; missing values cannot fall back to another mode. A consumer may offer an explicit System preference, mapping light to Day and dark to Night. Explicit choices take precedence; Aurora is never selected automatically.

## Colour

**Surfaces, from deep to high:** `bg-sunken` (terminal wells, code blocks, slider tracks) → `bg-base` (page, window, editor) → `bg-surface` (panels, sidebars, cards, inputs) → `bg-raised` (menus, dialogs, toasts) → `bg-hover` → `bg-pressed` (pressed) / `bg-selected` (neutral selection). In Night, `bg-hover` and `bg-pressed` are the two given seed colours oklch(32.95% 0.034 247) and oklch(35.00% 0.034 250).

**Text:** `ink` for content, `ink-muted` for secondary text, `ink-subtle` for timestamps, placeholders and line numbers, `ink-disabled` only for disabled controls. Do not invent greys in between.

**Lines:** `line` for every divider; `line-strong` for control borders (inputs, checkboxes, outline buttons) and scroll thumbs.

**Interaction:** `accent` is the single interactive colour - primary buttons, links, checked controls, focus, selection markers. Text on an `accent` fill is `on-accent` (dark in Night/Aurora, white in Day). Selected items use `accent-soft` fill with `accent-ink` text. Selection is always clearly marked: a fill plus a 2-3px `accent` marker or outline.

**The palette** (use as accents and for code; never as large fills):

| Token | Role | Seed / note |
| --- | --- | --- |
| `frost` | Info, types, links | Cool frost blue |
| `aurora` | Functions, Aurora accent | Teal |
| `lilac` | Keywords | Muted lilac |
| `sage` | Decorative green | Seed oklch(86.93% 0.028 130) |
| `fern` | Success, git added | Stronger sibling of sage |
| `latte` | Warning, numbers, ANSI yellow | Seed oklch(76.09% 0.065 71) |
| `foam` | Strings | Seed oklch(86.99% 0.028 46) |
| `rose` | Tags, git removed, ANSI red | Seed oklch(75.93% 0.066 10) |

**No yellow.** There is no true yellow anywhere. Where a platform demands one (ANSI yellow, "warning" in an IDE), use `latte`.

**Status:** `info`, `success`, `warning`, `danger` with `*-soft` backgrounds. `danger` is a stronger, redder rose of its own so it never blends with the latte `warning`. Status is never colour alone: pair it with an icon (`info`, `success`, `warning`, `danger` glyphs) or a word. Status colours stay inside the palette, so errors and git changes are visible without breaking the calm.

**Focus:** a 2px solid `focus-ring` (= `accent`) outline offset 2px from the control, at least 3:1 against every surface in every mode.

## Typography

- **Display** - Raleway, 600, with 700 for `title-4`: `display`, `title-1` … `title-4`. Headings and dialog titles only.
- **Text** - Noto Sans (Open Sans fallback): `lead`, `body` (15/24), `body-strong`, `small`, `label`, `caption`, `overline`. UI defaults to 14px (`text-md`); reading text to `body`.
- **Code** - Self-hosted Cascadia Code, upright and standard slanted italic, weights 200 to 700, falling back to Fira Code: `code` (14/22), `code-small`, `code-italic`. Ligatures default to on; handwritten cursive defaults to off. Users can change either preference independently. Fira's upright fallback retains readable comments.
- Sentence case for everything except `overline` (uppercase, +0.08em tracking).
- A little air: body line height 1.6, 16px default gap. Comfortable controls have a 40px minimum height; compact controls use 34px. Editors may go denser (13-14px); reading surfaces may use 17px `lead`.

The complete scale is in `semantic.typography` and `semantic.font-size`; size roles run from `text-2xs` (11px) to `text-3xl` (24px). [Font sources](../assets/fonts/README.md) include the original licences. Preserve readable fallbacks, allow text enlargement and keep long labels and paths wrapping.

## Space, shape and depth

- **Spacing** is a 4px grid: `step-1` (4) … `step-16` (64). `step-4` is the default gap; dialog and card padding is `step-5`.
- **Square windows, round controls.** Under `semantic.radius`, use `none` for window chrome and editor/terminal panes, `window` (2px) for floating windows, dialogs and menus, `panel` (4px) for cards, alerts and code blocks, `control` (6px) for inputs, and `full` for everything you press: buttons, toggles, tabs (pill), badges, switches, avatars.
- **Borders** are `border-hairline` (1px). Selected markers and focus are `border-active` / `border-focus` (2px).
- **Elevation:** `shadow-sm` (buttons), `shadow-raised` (cards), `shadow-overlay` (anything floating), `shadow-pressed` (wells). The scrim behind modals uses the `scrim` colour and its alpha value, without blur.

## Layout and interaction

Spacing follows the existing 4px grid. Reading text stays near 70 characters per line. Start with one column and preserve reading order; content determines whether additional columns fit.

| Rule | Value |
| --- | --- |
| Page gutter | 16px, increasing to 32px from 640px |
| Content maximum | 1200px |
| Breakpoints | 640px, 960px and 1280px |
| Comfortable density | 40px minimum control height, 16px gap, 20px panel padding |
| Compact density | 34px minimum control height, 12px gap and panel padding |
| Coarse pointer | At least 44px control target in either density |

These values live in the semantic layout, breakpoint and density roles. Use minimum heights so enlarged or translated text can grow. Compact gap and padding must not exceed comfortable values. Scrollable code and tables need a named, keyboard-accessible region.

Layers use `z-base` 0, `z-sticky` 10, `z-popover` 50, `z-modal` 60, `z-toast` 70 and `z-tooltip` 80. They order surfaces within the consuming platform's stacking rules; they do not implement focus or modality.

Preserve a 2px focus outline with a 2px offset. Keep focus visible and stable through updates. Selected items need a marker or outline as well as colour. Invalid fields need associated error text. Loading actions keep readable labels and suppress duplicate activation. Disabled text belongs only to unavailable controls, never help text or ordinary content. `tokens/usage.json` records the shared meanings, states and contrast pairs.

## Motion

Functional only. `duration-fast` (120ms) for hover, press and menus; `duration-base` (180ms) for expanding, switching and dialogs; `duration-slow` (260ms) for sheets, drawers and slides. Use `easing-standard` by default and `easing-exit` when a separate exit curve helps. Spinners and skeleton shimmer stop under `prefers-reduced-motion`. No auto-advancing carousels or parallax. Respect reduced-motion preferences by stopping decorative movement while retaining visible status text. In forced-colour environments, preserve boundaries and focus with the platform’s system colours; do not depend on shadows or colour alone.

## Logo

The mark shows steep blue cliffs around an open fjord, with a muted rose plane at the horizon. The full logo includes the supplied outlined Fjord wordmark.

- Use `fjord-logo.svg` and `fjord-mark.svg` on Night and Aurora. Use `fjord-logo-day.svg` and `fjord-mark-day.svg` on Day. `fjord-logo-on-dark.svg` includes a fixed Night background for standalone use.
- Keep the detailed mark at least 32 CSS pixels wide, including the mark portion of a full logo. Leave clear space of at least a quarter of the mark's displayed height on every side.
- Preserve the supplied paths, proportions and approved palette. Use the wordmark paths supplied with the logo.

The canonical files, Day token bindings and accessibility guidance are in [brand assets](../assets/brand/README.md).

## Iconography

Use Phosphor Fill in `currentColor`, with a 24 CSS pixel reference box. The foundation supplies 16 pinned SVGs with their original MIT notice. Sizes are 16px in controls, 18px in status/navigation, 20px for larger inline content and 24-28px for standalone or empty-state icons. Keep the icon size separate from its hit area. Hide decorative icons from assistive technology, and give meaningful standalone icons an accessible name. Use visible text for status. [Icon assets](../assets/icons/README.md) record provenance and update rules.

## Code and terminal

- Syntax colours stand out from the UI on purpose (Horizon-inspired, muted): keywords `syn-keyword` (lilac, italic), strings `syn-string` (foam), functions `syn-function` (aurora), types `syn-type` (frost), properties/tags `syn-property` (rose), numbers/constants `syn-number` (latte), operators `syn-operator`, regex/escapes `syn-regex`, punctuation `syn-punctuation`, comments `syn-comment` (grey-teal italic, never dim).
- The seven main syntax colours are kept at least 7 apart in perceptual colour difference (ΔE OKLab × 100) in every mode, and operators and regex at least 5, so a glance is enough to tell a string from a number. Re-check this whenever you change a syntax token.
- Diffs: `git-added`, `git-modified`, `git-removed` for gutter marks; `diff-add-bg`, `warning-soft`, `diff-del-bg` for added, changed and deleted line backgrounds. Use `ink` for text on these fills. Keep the `+ Added`, `~ Changed`, `- Deleted` markers visible. Search: `search-match` and `search-match-current`, with `ink` for matched text.
- The terminal uses `ansi-*` (16 colours) on `bg-base`, with the cursor in `accent` and selection in `selection`. A prompt may show context - path, git branch and status, runtime versions, duration - with the path in `frost`, branch in `lilac`, clean/dirty as `git-added`/`git-modified`, durations in `ink-subtle`.
- Dim or hint text uses `ansi-bright-black` on `bg-base` at full opacity, with at least 4.5:1 contrast in every mode. Do not apply extra dimming to autosuggestions or helper text.

Automatic checks enforce shared contrast and syntax colour distance. Terminal programs that choose their own colours, dimming or inverse video need implementation-specific checks. Solid `bg-sunken` or `bg-base` can provide a desktop background without images or gradients.

## Writing

Short, plain, sentence case. Verbs on buttons ("Apply theme", "Delete theme"). Errors say what happened and what to do. No exclamation marks, no emoji in UI text.

## Using the foundation

Use the [consumer guide](consumer-guide.md) to export design data and inherit themes. The foundation supplies generic CSS variables, resolved JSON and DTCG tokens; each implementation owns its controls, rendering and target-specific mappings.

Run `mise run check` after source changes. The [token contract](token-contract.md) defines validation and compatibility. A passing foundation check does not establish native rendering or accessibility support.
