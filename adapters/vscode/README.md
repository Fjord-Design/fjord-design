# Fjord for Visual Studio Code

Choose **Fjord Night**, **Fjord Day** or **Fjord Aurora** from **Preferences: Color Theme** after installing the local Fjord VSIX. Night is the dark blue default identity, Day is its light counterpart, and Aurora uses a deeper blue ground with teal accents. All three come from the shared Fjord semantic tokens.

This theme extension contains colour data and documentation. It has no executable extension entry point, activation events, dependencies, key bindings or configuration defaults. Installing it does not install fonts or replace your settings file. Selecting a theme through VS Code saves that theme preference through the editor's normal settings behavior.

## Install, upgrade and restore

In VS Code, run **Extensions: Install from VSIX…** and choose `fjord-themes-0.1.0.vsix`. Then run **Preferences: Color Theme** and select the desired Fjord mode. Keep a record of your previous colour theme, preferred light/dark themes and any custom colours so you can restore them.

For an isolated native test, launch VS Code with separate `--user-data-dir` and `--extensions-dir` paths and install the VSIX into that profile. This keeps normal user settings and installed extensions separate. Existing user colour customizations can override a theme; an isolated profile provides the clearest reference.

To upgrade, save the previous VSIX, install the replacement through **Install from VSIX…**, and reload if VS Code requests it. To undo the upgrade, install the saved version and restore the prior theme preference. To uninstall, choose another colour theme, uninstall **Fjord**, and restore any saved preferred-theme settings. No script in this extension edits or resets your settings.

The local publisher/name pair is `fjord.fjord-themes`. This artifact is not published to the Marketplace. Store publication is a separate release action. The package's `^1.145.0` engine requirement establishes its minimum VS Code version; the support matrix below records what was actually tested.

## Generated mapping and highlight behaviour

The adapter covers editor/workbench surfaces, inputs, buttons, lists, menus, focus, tabs, status, syntax, diagnostics, Git/diff, cursor/selection, find matches and the integrated 16-colour ANSI terminal. It supplies TextMate grammar rules and standard semantic-token rules. Semantic highlighting is enabled in the theme; the language service and the user's preferences determine whether semantic tokens are available.

Terminal selections use `semantic.color.selection` with the native `semantic.color.ink` foreground override. Standard `vs`/`vs-dark` editor themes retain syntax on selection: the pinned renderer applies `editor.selectionForeground` only in high-contrast themes, so this extension omits that ineffective field. The generator checks every emitted TextMate/semantic foreground on active and inactive editor selections. Night and Aurora retain their source selection fills, including Aurora's accepted teal (`#093b3b`). Day needs an explicit adapter-local mixture toward white to keep its retained comments and syntax readable; the report records this visibility tradeoff. Shared tokens and syntax foregrounds are unchanged.

Current and secondary editor find matches support native foreground overrides and use their source search roles with ink text. The source selection/search capabilities are pinned alongside the target's colour identifiers. Selection visibility is recorded descriptively and must also be assessed in the native application.

VS Code requires transparency for inactive selections and several decoration overlays. The generator adds target-specific alpha to the shared colour value and reports the actual emitted value beside the original token mapping. Secondary occurrence highlights use transparent fills and visible semantic accent borders, preserving syntax and underlying decorations. Terminal find has no separate foreground override: its fill alpha is reduced until retained ANSI text passes the emitted-colour checks, while the independent accent border remains visible. This avoids making selection or search nearly invisible merely to preserve the darkest ANSI hint colour.

Diff line and character fills are translucent. Their checks include the character fill layered over the line fill and editor background. Reports retain the source and actual emitted values, exact sRGB compositing, both unrounded and 8-bit HEX contrast, and all substitutions. Ordinary text requires 4.5:1; applicable focus, cursor, scrollbar and highlight-border indicators require 3:1. ANSI black in Night/Aurora remains an intentional palette slot, not ordinary readable text on a dark base; Day checks it as text.

The fixed target contract records colour identifiers from the VS Code 1.145.0 source at commit `a717861c07586a53d83b80d742cd3f59b20a6273`, with source URLs and SHA-256 values. Unknown colour IDs, unrecognized semantic selectors, wrong token types and target-version mismatches fail before output is written. Every semantic colour is mapped or explicitly unsupported. Third-party extensions can add scopes and colour IDs beyond this bounded mapping; those targets are not claimed as verified.

## Fonts and runtime limits

Fjord's font recommendations are Noto Sans for UI, Raleway for headings and Cascadia Code for code. A colour theme cannot install them or set font size, ligatures, cursor shape, window geometry, motion or keyboard behavior. Keep your current preferences, or select already installed fonts separately. Comments and keywords use ordinary italics; handwritten cursive is not requested.

Automated contrast checks measure emitted opaque colours and explicit sRGB overlay composition. Terminal minimum-contrast adjustments, SGR dim, truecolour, reverse video, font rendering, additional decorations and user customizations can change rendered pixels. These checks do not establish every terminal application's contrast, screen-reader output or every extension's semantic highlighting. Native inspection remains necessary.

| Target/check | Status |
| --- | --- |
| VS Code 1.145.0 colour identifiers and standard semantic tokens | Pinned source contract, offline target/type validation |
| Night, Day, Aurora and Fern reuse | Deterministic generation and automated contrast/mapping tests |
| VS Code 1.145.0 arm64 on macOS 26.7.1 | PASS: all three modes, TypeScript/JSON/shell, diagnostics, selection/search, Git diff, ANSI and selected terminal text; bounded fixtures recorded in the repository's XER-134 evidence |
| Other versions, operating systems, extension scopes and high-contrast modes | Not tested |

## Build from the Fjord repository

```sh
mise exec -- python3 adapters/vscode/generate.py
mise exec -- python3 scripts/package_vscode.py
```

The generator writes three theme JSON files, the extension manifest, README, MIT licence and a detailed `report.json` under `dist/adapters/vscode/`. The packager writes the local VSIX under `dist/release/`. Neither command installs the theme or publishes it. The diagnostic report remains a build artifact rather than executable extension content.

Use `--theme examples/themes/fern.tokens.json` to build the alternate identity without component edits. Repeated `--override` files keep the token core's precedence; the last file wins. `--root` accepts an absolute or relative project path. `--output-dir` changes the generated directory. `--help` is available offline. Invalid inputs, mappings or contrast exit with status 1 before output writes; invalid CLI arguments exit with status 2. Source and input files are protected from output replacement.

Source fixtures under `adapters/vscode/fixtures/` cover JSON keys, shell, TypeScript semantic tokens and a deliberate diagnostic, diff content, all sixteen ANSI slots, selection and search. Native fixtures do not require installing language servers: the target's built-in TypeScript/JSON support supplies those examples.

Fjord's theme, generator and fixtures use the bundled [MIT licence](LICENSE). The repository's pinned upstream contract evidence retains the VS Code MIT notice separately. Primary references: [colour theme format](https://code.visualstudio.com/api/extension-guides/color-theme), [theme colours](https://code.visualstudio.com/api/references/theme-color), [semantic highlighting](https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide), and [the pinned high-contrast selection guard](https://github.com/microsoft/vscode/blob/a717861c07586a53d83b80d742cd3f59b20a6273/src/vs/editor/browser/viewParts/viewLines/viewLine.ts#L125-L139).
