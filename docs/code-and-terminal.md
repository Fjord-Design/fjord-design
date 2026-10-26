# Code and terminal roles

Run `mise run preview`, then open `dist/code-and-terminal.html`. The specimen contains TypeScript, JSON, shell, terminal output, diff, diagnostics and Gitstatus. Switch between Night, Day and Aurora; use “View without colour” to inspect the status labels and markers.

The preview copies the self-hosted font bundle to `fonts/` beside the HTML. Keep both together for offline use. [Typography](typography.md) defines font roles, code ligatures, cursive preferences and original font licences.

The portable bindings are defined in `src/fjord_tokens/code_roles.py`. They reference existing `semantic.color.*` tokens. They do not introduce a second palette or change token names. Platform adapters should consume these bindings and document any unsupported features. The annotated examples live in `examples/code/samples.json`.

## Editor behaviour

Use `ink` on `bg-base` for unclassified code. Line numbers use `ink-subtle`, with `ink` for the current line number and `bg-surface` for the current line. A block cursor uses `accent` with `on-accent` text; a bar cursor uses `accent` and a visible solid shape. Do not lower cursor opacity.

Selected text uses `ink` on `selection`. Search results use `ink` on `search-match`, with an underline. The current result uses `search-match-current` and an `accent` outline. If a platform preserves syntax colours on selection or search fills, test those combinations separately before enabling that behaviour. The shared contract uses a single readable foreground for highlighted text.

Comments and keywords use italic when the selected font and platform support it. Italic must not change opacity. The standard foreground remains available when a platform cannot render italic. Syntax colour separates categories; punctuation, words and grammar still carry the code's meaning.

## Changes, diagnostics and Gitstatus

Diff lines use `ink` on the added, changed or deleted fill. Keep the signs and labels visible, including in monochrome. Gutter markers may use the matching Git colour. A changed-line marker describes an adapter's combined change view; a unified diff still uses its normal addition and deletion lines.

| Meaning | Required text or sign | Additional treatment |
| --- | --- | --- |
| Added line | `+ Added` | `diff-add-bg` fill |
| Changed line | `~ Changed` | `warning-soft` fill |
| Deleted line | `- Deleted` | `diff-del-bg` fill |
| Error | `E Error`, code and location | Wavy underline |
| Warning | `W Warning`, code and location | Dashed underline |
| Information | `I Info`, code and location | Solid underline |
| Hint | `H Hint`, code and location | Dotted underline |

Gitstatus uses `A Added`, `M Modified`, `D Deleted`, `R Renamed`, `? Untracked`, `! Ignored` and `U Conflict`. Keep renamed paths explicit. A conflict also needs a diagnostic message. Ignored files retain readable text; opacity is not a status signal. Terminal adapters may use plain labels where underline styles are unavailable.

Severity and change state must remain understandable without colour, following [WCAG 2.2 Use of Color](https://www.w3.org/TR/WCAG22/#use-of-color). The monochrome toggle is a visual review aid, not a simulation of every colour-vision condition or a full accessibility audit.

## ANSI and terminal behaviour

Use `ink` on opaque `bg-base`, `accent` for the cursor, `on-accent` inside a block cursor and `ink` on `selection`. Prompt roles cover path, branch, environment, duration, errors and the prompt symbol. Include words such as `branch` and `dev`, and preserve Git markers, so prompt context survives loss of colour.

Slots 0 to 7 are normal; slots 8 to 15 are their explicit bright entries. Foreground SGR codes 30 to 37 select normal entries and 90 to 97 select bright entries; background codes are 40 to 47 and 100 to 107. SGR 1 controls bold and SGR 2 controls faint intensity. See [XTerm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Character-Attributes).

Fjord keeps weight and palette selection separate. Prefer explicit bright codes and disable implicit bold-to-bright remapping when the target supports that setting. Where that setting is unavailable, document and test the target's behaviour. Bright entries may be darker in Day; never generate them by lightening the normal entries.

Slot 0 is a background role in Night and Aurora. Use slot 8, `ansi-bright-black`, for readable hint text and autosuggestions at full opacity. Do not apply SGR 2 or an opacity reduction to those hints. Slots 3 and 11 retain the yellow positions expected by applications, using muted café colours. Warning messages include a severity label.

The 16 values per mode are in the checked [platform tables](platforms.md). Arbitrary ANSI foreground/background combinations, inverse video, application-supplied truecolour, user overrides and terminal “dim” settings require separate testing. The shared tests cover the documented pairs. The browser specimen does not install a terminal theme; application configuration and tests belong to XER-116.

## Validation

`mise run check` validates the role bindings, contrast pairs, syntax separation and annotated examples. `mise run preview` repeats the relevant checks after applying any supplied theme and overrides. An invalid example or theme fails before replacing the HTML output.

Normal text requires 4.5:1. Comments on the editor background require 7.5:1 in Night and 5:1 in Day and Aurora. Cursor and focus indicators require 3:1 against their documented surfaces. The seven main syntax colours require a pairwise OKLab distance multiplied by 100 of at least 7; pairs involving operators or regex require at least 5. These separation thresholds are Fjord's design rules. The conversion follows [Björn Ottosson's OKLab definition](https://bottosson.github.io/posts/oklab/); it does not replace contrast or user testing.

Preview examples are static, explicitly annotated code specimens. They are not a general syntax highlighter. Theme buttons work with keyboard activation, code regions can scroll horizontally with focus, and diagnostic text wraps at narrow widths. Fonts use installed families and fallbacks; font distribution is a separate deliverable.

## Role bindings

The following table maps each portable binding to its semantic token. ANSI slots are included in the platform tables.

| Binding | Semantic token |
| --- | --- |
| `editor-background` | `semantic.color.bg-base` |
| `editor-text` | `semantic.color.ink` |
| `editor-current-line` | `semantic.color.bg-surface` |
| `editor-line-number` | `semantic.color.ink-subtle` |
| `editor-current-line-number` | `semantic.color.ink` |
| `editor-cursor` | `semantic.color.accent` |
| `editor-cursor-text` | `semantic.color.on-accent` |
| `editor-selection` | `semantic.color.selection` |
| `editor-selected-text` | `semantic.color.ink` |
| `editor-search` | `semantic.color.search-match` |
| `editor-current-search` | `semantic.color.search-match-current` |
| `editor-search-text` | `semantic.color.ink` |
| `diff-added-background` | `semantic.color.diff-add-bg` |
| `diff-changed-background` | `semantic.color.warning-soft` |
| `diff-deleted-background` | `semantic.color.diff-del-bg` |
| `diff-text` | `semantic.color.ink` |
| `diff-added` | `semantic.color.git-added` |
| `diff-changed` | `semantic.color.git-modified` |
| `diff-deleted` | `semantic.color.git-removed` |
| `diagnostic-error` | `semantic.color.danger` |
| `diagnostic-warning` | `semantic.color.warning` |
| `diagnostic-info` | `semantic.color.info` |
| `diagnostic-hint` | `semantic.color.ink-muted` |
| `git-added` | `semantic.color.git-added` |
| `git-modified` | `semantic.color.git-modified` |
| `git-deleted` | `semantic.color.git-removed` |
| `git-renamed` | `semantic.color.info` |
| `git-untracked` | `semantic.color.ink-muted` |
| `git-ignored` | `semantic.color.git-ignored` |
| `git-conflict` | `semantic.color.danger` |
| `terminal-background` | `semantic.color.bg-base` |
| `terminal-text` | `semantic.color.ink` |
| `terminal-cursor` | `semantic.color.accent` |
| `terminal-cursor-text` | `semantic.color.on-accent` |
| `terminal-selection` | `semantic.color.selection` |
| `terminal-selected-text` | `semantic.color.ink` |
| `terminal-hint` | `semantic.color.ansi-bright-black` |
| `prompt-path` | `semantic.color.frost` |
| `prompt-branch` | `semantic.color.lilac` |
| `prompt-environment` | `semantic.color.aurora` |
| `prompt-duration` | `semantic.color.ink-subtle` |
| `prompt-symbol` | `semantic.color.accent` |
| `prompt-error` | `semantic.color.danger` |
| `syntax-comment` | `semantic.color.syn-comment` |
| `syntax-keyword` | `semantic.color.syn-keyword` |
| `syntax-string` | `semantic.color.syn-string` |
| `syntax-number` | `semantic.color.syn-number` |
| `syntax-type` | `semantic.color.syn-type` |
| `syntax-function` | `semantic.color.syn-function` |
| `syntax-property` | `semantic.color.syn-property` |
| `syntax-variable` | `semantic.color.syn-variable` |
| `syntax-operator` | `semantic.color.syn-operator` |
| `syntax-punctuation` | `semantic.color.syn-punctuation` |
| `syntax-regex` | `semantic.color.syn-regex` |
| `syntax-text` | `semantic.color.ink` |
