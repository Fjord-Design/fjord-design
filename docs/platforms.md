# Platforms

How Fjord's tokens map onto the places you will build themes for. Use the token names, not the hex values, in any generator: values are resolved per mode from the DTCG sources in `tokens/` (Night, Day, Aurora). Short colour names below map to `semantic.color.<name>`. Run `mise run validate` to check both colour tables.

Night is the default when no theme is selected. Every colour token must define a value for all three themes. Resolve aliases within the selected theme. Reject missing values or unresolved aliases and report the token and theme; never substitute Night's value.

The [code and terminal specification](code-and-terminal.md) defines cursor, line-number, diff, diagnostic and prompt bindings, including the text markers required alongside colour. Run `mise run preview` to inspect all three modes in a browser. These mappings describe adapter requirements; terminal installation and platform support require separate application tests.

The MVP terminal target is WezTerm 20240203-110809-5046fc22 on macOS. After MVP, the first editor target is Zed 1.25.3, followed by VS Code and Neovim. The [architecture contract](decisions/0001-platform-boundaries.md) defines version evidence, browser targets, accessibility acceptance and the limits of each adapter.

## Core values at a glance

| Token | Night | Day | Aurora |
| --- | --- | --- | --- |
| `bg-base` | `#111920` | `#ecf1f5` | `#080e16` |
| `bg-surface` | `#192530` | `#fdfeff` | `#0f1923` |
| `bg-raised` | `#1e2b38` | `#ffffff` | `#131f2c` |
| `bg-sunken` | `#0b1217` | `#e0e8ed` | `#04080e` |
| `line` | `#24323f` | `#d3dbe1` | `#1a2734` |
| `ink` | `#e1e9ef` | `#16232f` | `#e5ecf1` |
| `ink-muted` | `#a7b5c0` | `#435260` | `#a8b6c1` |
| `ink-subtle` | `#95a4b1` | `#52616f` | `#92a1ae` |
| `accent` | `#90c6e3` | `#1c6391` | `#7ad7d1` |
| `on-accent` | `#111920` | `#f9fcfe` | `#080e16` |
| `selection` | `#1a4159` | `#b3d6ee` | `#093b3b` |
| `syn-comment` | `#90afaa` | `#636572` | `#90afaa` |
| `syn-keyword` | `#c2acde` | `#694b96` | `#c6aee4` |
| `syn-string` | `#e5cfc5` | `#88341f` | `#e9cec1` |
| `syn-function` | `#83d3be` | `#12695c` | `#81dfca` |
| `syn-type` | `#92c1e3` | `#215e92` | `#93c9ec` |
| `syn-property` | `#de99a9` | `#9b4251` | `#e19ba5` |
| `syn-number` | `#ccab84` | `#7a511f` | `#d7af83` |

## Terminal (Windows Terminal, Alacritty, WezTerm, Kitty, Ghostty, iTerm2)

| Slot | Token | Notes |
| --- | --- | --- |
| background | `bg-base` | Opaque: set opacity to 100% and disable acrylic/blur. |
| foreground | `ink` | |
| cursor | `accent` | Block or bar; cursor text `on-accent`. |
| selection background | `selection` | Selection foreground stays `ink`. |
| tab bar / title bar | `bg-sunken`, active tab `bg-base` with a 2px `accent` top edge | Square corners (`radius-none`). |
| padding | `space-3`…`space-4` | A little air around the text. |
| font | `font-mono` at 14px, ligatures on | Cascadia Code, fallback Fira Code. |

**ANSI 16**

| # | Token | Night | Day | Aurora |
| --- | --- | --- | --- | --- |
| 0 | `ansi-black` | `#1e2b38` | `#16232f` | `#131f2c` |
| 1 | `ansi-red` | `#d7a0a7` | `#9b4251` | `#e19ba5` |
| 2 | `ansi-green` | `#a0cba0` | `#326635` | `#9fd4a5` |
| 3 | `ansi-yellow` | `#ccab84` | `#7a511f` | `#d7af83` |
| 4 | `ansi-blue` | `#92c1e3` | `#215e92` | `#93c9ec` |
| 5 | `ansi-magenta` | `#c2acde` | `#694b96` | `#c6aee4` |
| 6 | `ansi-cyan` | `#83d3be` | `#12695c` | `#81dfca` |
| 7 | `ansi-white` | `#b3c0ca` | `#606a74` | `#b6c3cd` |
| 8 | `ansi-bright-black` | `#788795` | `#5f6d7a` | `#6e7c8b` |
| 9 | `ansi-bright-red` | `#e8b5ba` | `#873041` | `#f0b0b9` |
| 10 | `ansi-bright-green` | `#d9e7cf` | `#38572d` | `#d2e9cd` |
| 11 | `ansi-bright-yellow` | `#e0c29d` | `#754a22` | `#e5c49c` |
| 12 | `ansi-bright-blue` | `#aad9f2` | `#00527e` | `#a8ddf6` |
| 13 | `ansi-bright-magenta` | `#d2c7ee` | `#593e88` | `#d4c5f3` |
| 14 | `ansi-bright-cyan` | `#9ee4dd` | `#005c5a` | `#9cede3` |
| 15 | `ansi-bright-white` | `#edf3f7` | `#111920` | `#f2f6f9` |

Dim or hint text and autosuggestions use `ansi-bright-black` on `bg-base` at full opacity, with at least 4.5:1 contrast in every theme. Do not apply extra dimming to this text.

Normal and bright are separate palette entries. Select bright colours explicitly; bold changes weight. Bright colours can be darker in Day. Keep yellow in slots 3 and 11, using the café tones shown above. Do not derive the bright palette by lightening normal colours.

**Prompt (Starship, Oh My Posh, Powerlevel10k):** path `frost`, git branch `lilac`, clean `git-added`, dirty `git-modified`, conflicts `git-removed`, runtime versions `aurora`, command duration and time `ink-subtle`, error exit code `danger`, the prompt character `accent`. Separators are plain characters, no powerline blocks of colour.

## Code editors (VS Code, JetBrains, Zed, Neovim, Helix)

| Editor surface | Token |
| --- | --- |
| editor background | `bg-base` |
| gutter, line numbers | `bg-base`, `ink-subtle`; current line number `ink` |
| current line highlight | `bg-surface` |
| selection / word highlight | `selection` / `bg-active` |
| find match / current match | `search-match` / `search-match-current` |
| side bar, panels, status bar, activity bar | `bg-surface` with `line` borders |
| title bar, tab strip | `bg-sunken`; active tab `bg-base` with 2px `accent` top border; inactive tab text `ink-muted` |
| widgets, hovers, suggest, command palette | `bg-raised` + `shadow-overlay`, border `line` |
| focus border, buttons, badges | `accent`, text `on-accent` |
| errors / warnings / info / hints | `danger` / `warning` / `info` / `ink-muted`; severity labels and underlines as defined in the code specification |
| git decorations | `git-added`, `git-modified`, `git-removed`, `git-ignored` |
| diff editor | Added `diff-add-bg`, changed `warning-soft`, deleted `diff-del-bg`; text `ink`, with `+ Added`, `~ Changed`, `- Deleted` markers |
| brackets / indent guides | `ink-subtle` / `line`; active guide `line-strong` |
| terminal | the ANSI table above |

**TextMate / tree-sitter scopes:** `comment` → `syn-comment` italic · `keyword`, `storage`, `@keyword` → `syn-keyword` italic · `string`, `@string` → `syn-string` · `entity.name.function`, `support.function`, `@function` → `syn-function` · `entity.name.type`, `support.class`, `@type`, `@constructor` → `syn-type` (but Lua table braces, `@constructor.lua`, → `syn-punctuation`) · `variable`, `@variable` → `syn-variable` · `variable.other.property`, `entity.other.attribute-name`, `entity.name.tag`, `@property`, `@tag` → `syn-property` · `constant.numeric`, `constant.language`, `@number`, `@boolean`, `@constant` → `syn-number` · `keyword.operator`, `@operator` → `syn-operator` · `punctuation`, `@punctuation` → `syn-punctuation` · `string.regexp`, `constant.character.escape`, `@string.regexp`, `@string.escape` → `syn-regex` · `markup.heading` → `syn-type` bold · `markup.inserted` / `markup.deleted` → `git-added` / `git-removed`.

For Neovim, also map `Normal` → `ink` on `bg-base`, `NormalFloat` → `bg-raised`, `CursorLine` → `bg-surface`, `Visual` → `selection`, `Search` / `IncSearch` → `search-match` / `search-match-current`, `DiagnosticError/Warn/Info/Hint` → `danger`/`warning`/`info`/`ink-muted`.

## Desktop (KDE Plasma, GNOME, Windows)

- Window background `bg-base`, view/base `bg-surface`, alternate rows `bg-sunken`, tooltips `bg-inverse` with `ink-inverse`, selection `accent` with `on-accent`, focus/hover decoration `accent`, links `accent`, visited links `lilac`, negative/neutral/positive text `danger`/`warning`/`success`.
- Window decorations: square corners, title bar `bg-sunken`, active window marked by `ink` title text and an `accent` hairline under the title bar; inactive windows `ink-subtle`. No transparency or blur effects.
- Panels/taskbar: `bg-sunken`, active task underlined in `accent`.
- Wallpaper: solid `bg-sunken` (Night/Aurora) or `bg-base` (Day).
- Fonts: UI Noto Sans 10pt, titles Raleway 600, monospace Cascadia Code 11pt.
- Windows 11: accent colour = Night `accent`, dark mode for apps; turn off transparency effects.

## Browser and web

- The current web reference exposes scoped CSS custom properties through `ThemeScope`, using `data-fjord-mode`, `data-fjord-preference` and `data-fjord-identity`. Night is the default; explicit System follows `prefers-color-scheme` for Night/Day, and Aurora is a user choice. General CSS exports provide `--fjord-token-*` properties for other integrations. See the [theme guide](theme-guide.md) for both paths.
- For styling sites you visit (Stylus, Dark Reader custom theme): background `bg-base`, text `ink`, selection `selection`, links `accent`, scrollbar thumb `line-strong`.
- Browser chrome themes (Firefox `manifest.json` theme, Chrome theme): frame `bg-sunken`, toolbar and active tab `bg-base`, tab text `ink`, inactive tab text `ink-muted`, URL bar `bg-surface`, popups `bg-raised`, accent `accent`.

## Games and launchers

Steam, Discord, Battle.net and similar skins: use Aurora. Ground `bg-base`, panels `bg-surface`, primary buttons `accent`, online status `success`, in-game `aurora`, away `warning`, do-not-disturb `danger`.
