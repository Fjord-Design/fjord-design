# Target support and ownership

Recorded 8 November 2026. Xerrion is the repository maintainer and release owner for the shared contract and each adapter below. Ownership does not imply vendor support. Night, Day and Aurora are generated for every adapter. A result applies only to its named target and scope.

| Consumer | Target observed | Evidence and limits | Owner |
| --- | --- | --- | --- |
| Python token core | Python 3.15.0; contract 0.3.0 | DTCG, alias, composition, source/HEX contrast and deterministic-output checks. Independent of Svelte and native runtimes. | Xerrion |
| Svelte reference | Svelte 5.57.2, TypeScript 7.0.2, local in-app Chromium browser | Exact 66 exports; all 19 family/composition views in 3 modes × Fjord/Fern/Work/Gaming; selected keyboard, focus, form and accessibility-tree checks. No current full cross-browser matrix or spoken screen-reader result. | Xerrion |
| WezTerm | 20260716-195552-76b606ec; CachyOS Linux, KDE Plasma 6.8.2 Wayland session using isolated XWayland windows | Fjord/Work/Gaming × 3 modes: native palette, truecolour, selection, fixture text/links/diff/errors and screenshots. Native Lua preserves unrelated settings. Does not establish macOS or native Wayland rendering. Fonts, ligatures and shell behaviour remain user owned. | Xerrion |
| Zed | 1.27.0 on macOS 26.7.1 for base Fjord | Previously accepted native base-theme result in [XER-140](verification/xer-140.md). Work/Gaming generated/schema/contrast checks only. Native new variants and original 1.25.3 target not run. Retained-text highlight fallbacks apply. | Xerrion |
| VS Code | 1.145.0 arm64 on macOS 26.7.1 for base Fjord | Previously accepted native base-theme result in [XER-134](verification/xer-134.md). Work/Gaming VSIX and themes validated locally; native new variants not run. Target-selected alpha/fallback limits remain. | Xerrion |
| Neovim | 0.12.6 is the intended target; no runtime available | Generated highlights, Treesitter/LSP roles, ANSI, opaque surfaces, contrast and fixture checks only. Owner chose generation without installation. No tested third-party plugin integration or native support claim. | Xerrion |
| KDE/Qt proof of concept | Plasma 6.8.2, Qt 6.12.0, Breeze, Noto Sans on CachyOS Linux | Nine isolated Qt Widgets palettes/focus/selection previews, before/after screenshots and checked settings hashes. Limited to application colours and selected surfaces. Rich-text links, inactive/disabled rendering and style-specific effects can differ. No whole-desktop, Plasma shell, KWin, GTK or Electron support claim. | Xerrion |

All adapters emit explicit mapping, unsupported-role and fallback reports. Fonts, radius, shadows, animation and interaction do not become portable merely because colour roles map. The [adapter guides](adapters.md) document installation and reversal.

## Upgrade verification

For each target update, record the exact application, OS, display backend, fonts and package version. Build into a new directory; run mapping/contrast tests and the applicable existing-runtime check. Recheck normal and bright ANSI, cursor, selection, comments/italics, links, diagnostics, diff, dark/light switching and restoration. For web controls, also recheck keyboard paths, focus, error associations, enlarged text, reduced motion and the intended screen reader.

`mise run ci` covers deterministic generation and package/type checks. `mise run test:wezterm` executes nine native Lua configurations with success markers; a zero exit code alone is insufficient because WezTerm can fall back after a configuration error. `mise run test:kde` builds and captures an isolated Qt application using existing tools. `mise run test:neovim` needs an existing permitted Neovim. Missing native tools report NOT RUN; obtain permission before installing anything.

`adapters/wezterm/check_native.py` captures owned native windows and checks palette pixels, truecolour and native selection. `scripts/check_kde.py` compares nine native QPalettes with generated RGB values and records focus, selection and screenshots. Compare captures against a reviewed baseline with `mise exec -- python3 scripts/compare_native_images.py BASELINE ACTUAL`. It checks exact PNG membership and changed-pixel counts; run it on the same OS/font/toolchain. Review platform updates before accepting a changed baseline. These display-dependent checks are separate from headless CI and are not silently counted as passes there.

New frameworks, mobile platforms, desktop shells and third-party plugin integrations require separately scoped adapter work and target evidence. KDE follow-ups and its bounded production decision are in the [proof-of-concept guide](../adapters/kde/README.md).
