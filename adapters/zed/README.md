# Fjord for Zed

Build Night, Day and Aurora from the shared Fjord tokens:

```sh
mise exec -- python3 adapters/zed/generate.py
```

The command validates the source project, resolves the declarative [mapping](adapter.json), checks emitted colours and writes `dist/adapters/zed/themes/fjord.json`, `report.json`, `extension.toml` and `LICENSE`. It does not install a theme, download fonts or edit Zed settings. Repeated builds from the same inputs produce the same bytes.

Use `--theme examples/themes/fern.tokens.json` for the alternate identity. Repeated `--override` files keep the core's precedence: the last file wins. `--output-dir` changes the destination. Invalid mappings, unknown schema keys, unreadable inputs or failed contrast checks exit with status 1 before generated output is written; invalid CLI arguments exit with status 2. `--help` is available offline. Outputs cannot replace source or input files.

## Install, choose and restore

For a first installation on macOS, copy the generated theme file into Zed's themes directory. Zed on macOS uses `$HOME/.config/zed`, independently of `XDG_CONFIG_HOME`. The interactive copy asks before replacing an existing file:

```sh
mkdir -p "$HOME/.config/zed/themes"
cp -i dist/adapters/zed/themes/fjord.json "$HOME/.config/zed/themes/fjord.json"
```

On Linux, the equivalent directory is `${XDG_CONFIG_HOME:-$HOME/.config}/zed/themes`. Use that directory for both commands if testing on Linux; Linux application support remains unverified. A Zed instance launched with `--user-data-dir` instead reads themes from its own `config/themes` directory.

In Zed, run **theme selector: toggle** from the command palette and choose **Fjord Night**, **Fjord Day** or **Fjord Aurora**. Keep your prior theme choice and a copy of your own settings before saving a new default. The theme generator does not set a default. To follow system appearance, choose Day for light and Night for dark in your own theme preferences; Aurora is an explicit choice.

Before upgrading, save any existing `fjord.json` outside the themes directory, regenerate, then use the same copy command and confirm replacement. To uninstall, select another theme first and remove only the installed `fjord.json`. Restore the saved theme file and your previous theme preference if you want to undo the upgrade. No other settings need to be replaced.

The generated extension manifest supports a separately reviewed extension package. Store publication is a separate release action; nothing is published by this task.

## Mapping and retained text

The adapter maps workspace and editor surfaces, text and icons, focus, line numbers, the local cursor/selection, syntax, diagnostics, Git/diff and the integrated ANSI terminal. The mapping report lists all targets with source roles and provenance. Every semantic colour is either mapped or explicitly unsupported; other token families are covered by the manifest's capability fallbacks.

Zed shares the local selection and search fills between the editor and terminal, retaining syntax and ANSI foregrounds. Fjord's portable fills assume that selected text changes to `ink`; copying those fills directly makes retained foregrounds fail contrast. For dark themes the adapter mixes each constrained HEX fill toward black; for Day it mixes toward white. It selects the first passing step in a fixed 1,024-step search after 8-bit rounding. The same conservative foreground check also covers the three editor-only document-highlight fills. Syntax colours and shared token values are unchanged.

Native feedback found Aurora's first constrained selection fill (`#031111`) too close to its base (`#080e16`). An Aurora-only fallback now emits the existing `ink-subtle` role (`#92a1ae`) for terminal bright-black and the nine dim/hint targets. This lets the shared selection retain its source teal (`#093b3b`) while every applicable opaque foreground still passes 4.5:1. The fill/base ratio rises from 1.00684:1 to 1.56754:1; this comparison measures the change and is not a WCAG pass threshold. Night and Day keep their original foreground mappings. `report.json` retains each original mapping value and role, adds the actual `emittedValue`, and names every foreground/fill substitution, its fallback role and reason.

Emitted ordinary editor text, syntax and applicable terminal foregrounds are checked at 4.5:1 on their base/active-line surfaces and five highlight fills. Line numbers are checked on the editor surfaces, outside the selected text region. Ordinary editor text and syntax are also checked on created, deleted and modified diff/status fills. ANSI black in Night/Aurora remains the intentional palette slot and is not claimed as readable ordinary text on a dark background; Day checks it as text. Bright black, dim/hint colours and every other ANSI slot remain checked as emitted opaque colours.

These solid-HEX checks do not verify every pixel composited by Zed. The running Zed source [fades unfocused editor selections to 50% opacity](https://github.com/zed-industries/zed/blob/c84af5b99cbc366d675064e52b23664a015b6e35/crates/editor/src/element.rs#L962-L965), may adjust editor text using its APCA contrast setting, and renders terminal SGR DIM text with reduced opacity. Native inspection remains necessary for focused/unfocused selection visibility and dimmed terminal text. The adapter does not change these runtime settings.

The pinned v0.2.0 schema exposes one `search.match_background` and no separate current-match or selection-border colour. The current match therefore follows Zed's native navigation/cursor behavior. Diagnostic icons, underlines and diff signs are owned by Zed; themes alone do not establish accessible interaction. Remote collaboration colors are outside this local-player mapping.

## Fonts and other platform limits

Zed theme JSON controls colours and syntax emphasis. It cannot set Noto Sans, Raleway, Cascadia Code, font sizes, ligatures, radius, shadow geometry or reduced-motion preferences. Select installed fonts and ligature preferences in your own Zed settings. The adapter emits ordinary italic comments/keywords and opaque surfaces. It does not install fonts or alter keyboard behavior, cursor shape, app geometry or user preferences. The manifest explains each fallback.

## Verification and support

| Target | Status |
| --- | --- |
| Official Zed theme schema v0.2.0, pinned 6 November 2026 | Generated fields/types and opaque HEX validated offline; schema checksum verified |
| Mapping, deterministic builds, Fern reuse and emitted contrast | Automated tests in `tests/test_zed_adapter.py` |
| Zed 1.27.0 (`c84af5b`) on macOS 26.7.1 | Native Night/Day/Aurora syntax, diagnostics, diff, cursor, selection/search and labelled ANSI fixtures checked; settings restored |
| Zed 1.26.2 bundle metadata | Reported on disk; not the running version observed during native tests |
| Original Zed 1.25.3 target | Not installed or tested in this delivery |
| Other versions, operating systems and remote collaborators | Not tested |

Use [the native fixtures](fixtures/) to inspect JSON, shell and TypeScript, the deliberate TypeScript diagnostic, Git/diff signs, selected text, search matches, local cursor and all sixteen terminal slots. Run `sh adapters/zed/fixtures/ansi.sh` in Zed's terminal. Exact native results belong in [XER-140 verification](../../docs/verification/xer-140.md); automated schema and colour checks do not establish application support.

The theme, generator and fixtures use the [Fjord MIT licence](LICENSE). The unmodified official schema has its own upstream [GPL-3.0-or-later licence](schema/LICENSE-GPL); [provenance](schema/provenance.json) records its URL, SHA-256, retrieval date and defining package licence. The schema is used for offline validation and is not copied into the generated theme bundle. Sources: [Zed theme extensions](https://zed.dev/docs/extensions/themes) and [official theme schema](https://zed.dev/schema/themes/v0.2.0.json).
