Fjord is a calm, cool, Nordic theme system for every surface you sit in front of: editor, terminal, desktop, browser, the sites you build and the games you play. It is a quiet workspace by day and a personal room at night. When rules conflict, **calm wins** - over density, over personality, over novelty.

## Principles

- **Calm first.** Colour is an accent, never a surface. Large areas are always one of the `bg-*` blues; the café and Nordic hues appear in small doses: a button, a badge, a line of code.
- **Cool ground, warm sparks.** The ground is a cool dark blue (`bg-base`, seeded from oklch(20.88% 0.018 245)). Warmth comes from café tones (`latte`, `foam`) and `rose`, kept muted so they glow instead of shout.
- **Discreet, but recognisably yours.** No neon, no synthwave gradients, no baroque ornament, no shabby texture. Personality lives in the combination: frost-blue interaction, café-toned code, square windows with round buttons.
- **Clear contrast.** Every text token meets 4.5:1 on the surfaces its note names, in every theme. Comments in code are content: `syn-comment` holds 7.5:1 on `bg-base` in Night.
- **Depth, not glass.** Floating things get `shadow-raised` or `shadow-overlay` with a faint top highlight. Never transparency or backdrop blur in panels, terminals or windows.
- **Few frames.** Separate regions with 1px hairlines (`line`) and fill changes (`bg-base` → `bg-surface`), not boxes inside boxes. The one coloured edge Fjord allows is the 3px `accent` marker on the current item in navigation (Sidebar); never on cards or alerts.

## Themes and moods

| Theme | id | When | Character |
| --- | --- | --- | --- |
| Night | `night` | Default. Home office, evenings, anywhere dim. | Cool dark blue, frost-blue `accent`. |
| Day | `day` | Bright offices and daylight. | Pale blue-grey ground, deep blue `accent`; every hue darkened to stay legible. |
| Aurora | `aurora` | Gaming and late nights. | Deeper, almost-black blue ground; aurora-teal `accent`; the palette a touch brighter. |

Night is the default when no theme is selected. Every colour token must define a value for Night, Day and Aurora. Aliases resolve within the selected theme. Generators must reject missing values or unresolved aliases and report the token and theme; they must never substitute Night's value. All three share every token name, so a theme built for one platform gets all three moods by swapping the value column. Follow the OS light/dark setting with Day ↔ Night; offer Aurora as an explicit choice (a "gaming" profile).

## Colour

**Surfaces, from deep to high:** `bg-sunken` (terminal wells, code blocks, slider tracks) → `bg-base` (page, window, editor) → `bg-surface` (panels, sidebars, cards, inputs) → `bg-raised` (menus, dialogs, toasts) → `bg-hover` → `bg-active` (pressed, neutral selection). In Night, `bg-hover` and `bg-active` are the two given seed colours oklch(32.95% 0.034 247) and oklch(35.00% 0.034 250).

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

**Focus:** a 2px solid `focus-ring` (= `accent`) outline offset 2px from the control, at least 3:1 against every surface in every theme.

## Typography

- **Display** - Raleway, 600, with 700 for `type-title-4`: `type-display`, `type-title-1` … `type-title-4`. Headings and dialog titles only.
- **Text** - Noto Sans (Open Sans fallback): `type-lead`, `type-body` (15/24), `type-body-strong`, `type-small`, `type-label`, `type-caption`, `type-overline`. UI defaults to 14px (`text-md`); reading text to `type-body`.
- **Code** - Self-hosted Cascadia Code, upright and standard slanted italic, weights 200 to 700, falling back to Fira Code: `type-code` (14/22), `type-code-small`, `type-code-italic`. Ligatures default to on; handwritten cursive defaults to off. Users can change either preference independently. Fira's upright fallback retains readable comments.
- The typography stylesheet maps roles to `.fjord-type-*` classes, including `.fjord-type-body`. See [the scale, font assets and import contract](typography.md). For a size on its own, use the `text-*` scale: `text-2xs` (11) … `text-3xl` (24).
- Sentence case for everything except `overline` (uppercase, +0.08em tracking).
- A little air: body line height 1.6, controls 34px tall, 16px default gap. Editors may go denser (13-14px); web pages may go roomier (17px `lead`).

## Space, shape and depth

- **Spacing** is a 4px grid: `space-1` (4) … `space-16` (64). `space-4` is the default gap; dialog and card padding is `space-5`.
- **Square windows, round controls.** `radius-none` for window chrome and editor/terminal panes, `radius-window` (2px) for floating windows, dialogs and menus, `radius-panel` (4px) for cards, alerts and code blocks, `radius-control` (6px) for inputs, and `radius-full` for everything you press: buttons, toggles, tabs (pill), badges, switches, avatars.
- **Borders** are `border-hairline` (1px). Selected markers and focus are `border-active` / `border-focus` (2px).
- **Elevation:** `shadow-sm` (buttons), `shadow-raised` (cards), `shadow-overlay` (anything floating), `shadow-pressed` (wells). The scrim behind modals uses the `scrim` colour and its alpha value, without blur.

## Motion

Functional only. `duration-fast` (120ms) for hover, press and menus; `duration-base` (180ms) for expanding, switching and dialogs; `duration-slow` (260ms) for sheets, drawers and slides. Use `easing-standard` by default and `easing-exit` when a separate exit curve helps. Spinners and skeleton shimmer stop under `prefers-reduced-motion`. No auto-advancing carousels or parallax. [Foundation rules](foundations.md) define density, breakpoints, target sizes, layer order, reduced motion and forced colours.

## Logo

The mark shows steep blue cliffs around an open fjord, with a muted rose plane at the horizon. The full logo includes the supplied outlined Fjord wordmark.

- Use `fjord-logo.svg` and `fjord-mark.svg` on Night and Aurora. Use `fjord-logo-day.svg` and `fjord-mark-day.svg` on Day. `fjord-logo-on-dark.svg` includes a fixed Night background for standalone use.
- Keep the detailed mark at least 32 CSS pixels wide, including the mark portion of a full logo. Leave clear space of at least a quarter of the mark's displayed height on every side.
- Preserve the supplied paths, proportions and approved palette. Use the wordmark paths supplied with the logo.

The canonical files, Day token bindings and accessibility guidance are in [brand assets](../assets/brand/README.md).

## Iconography

Use Phosphor Fill in `currentColor`, with a 24 CSS pixel reference box. The Svelte `Icon` primitive uses 16 pinned SVGs with their original MIT notice. Sizes are 16px in controls, 18px in status/navigation, 20px for larger inline content and 24-28px for standalone or empty-state icons. [Foundation guidance](foundations.md) defines alignment and accessible names; [icon assets](../assets/icons/README.md) records provenance and update rules.

## Code and terminal

- Syntax colours stand out from the UI on purpose (Horizon-inspired, muted): keywords `syn-keyword` (lilac, italic), strings `syn-string` (foam), functions `syn-function` (aurora), types `syn-type` (frost), properties/tags `syn-property` (rose), numbers/constants `syn-number` (latte), operators `syn-operator`, regex/escapes `syn-regex`, punctuation `syn-punctuation`, comments `syn-comment` (grey-teal italic, never dim).
- The seven main syntax colours are kept at least 7 apart in perceptual colour difference (ΔE OKLab × 100) in every theme, and operators and regex at least 5, so a glance is enough to tell a string from a number. Re-check this whenever you change a syntax token.
- Diffs: `git-added`, `git-modified`, `git-removed` for gutter marks; `diff-add-bg`, `warning-soft`, `diff-del-bg` for added, changed and deleted line backgrounds. Use `ink` for text on these fills. Keep the `+ Added`, `~ Changed`, `- Deleted` markers visible. Search: `search-match` and `search-match-current`, with `ink` for matched text.
- The terminal uses `ansi-*` (16 colours) on `bg-base`, with the cursor in `accent` and selection in `selection`. A prompt may show context - path, git branch and status, runtime versions, duration - with the path in `frost`, branch in `lilac`, clean/dirty as `git-added`/`git-modified`, durations in `ink-subtle`.
- Dim or hint text uses `ansi-bright-black` on `bg-base` at full opacity, with at least 4.5:1 contrast in every theme. Do not apply extra dimming to autosuggestions or helper text.
- Wallpaper and desktop: a solid `bg-sunken` (or `bg-base`) colour - no images, no gradients.

The complete bindings and non-colour markers are in [code and terminal roles](code-and-terminal.md). Run `mise run preview` for the interactive examples. Automatic checks enforce both contrast and syntax colour distance.

## Writing

Short, plain, sentence case. Verbs on buttons ("Apply theme", "Delete theme"). Errors say what happened and what to do. No exclamation marks, no emoji in UI text.

## Using this system

The repository delivers the token core, code specimen, canonical SVG brand assets and verified self-hosted fonts. Svelte components and the Storybook catalogue are delivered through the MVP tasks. [Typography verification](verification/xer-105.md) records font, licence, fallback and browser checks.

- Web adapters should expose tokens as CSS custom properties: `var(--bg-base)`, `var(--space-4)`, `var(--radius-full)`, `var(--font-mono)`. Set `data-theme="night" | "day" | "aurora"` on `<html>`; with no attribute you get Night.
- The web reference uses Svelte + TypeScript, Vite and Storybook. Components consume generated token output and have framework-independent specifications. The [architecture decision](decisions/0001-platform-boundaries.md) defines boundaries and distribution; the [dependency policy](dependency-policy.md) defines stable-version selection and the TypeScript 7 bridge.
- Building a theme for another app (VS Code, terminals, JetBrains, Neovim, KDE/GNOME, Windows, browsers): read the **Platforms** section for which token goes where. Values per mode are resolved from the DTCG sources in `tokens/`; see [platform mappings](platforms.md).

## Validation

Use `mise run check`. The format and validation rules are in [the token contract](token-contract.md). Release rules are in [versions and token lifecycle](versioning.md).
