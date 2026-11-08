# KDE Plasma colour investigation

The owner selected KDE Plasma 6.8.2 on 8 November 2026. This proof of concept covers Qt application colours, text, selection, native focus and opaque surfaces using the installed Qt 6.12.0/Breeze stack on CachyOS Linux. It does not apply a global theme or claim Plasma panel, KWin decoration, GTK, Electron or cross-OS support.

Generate the three `.colors` files with `mise run adapters:kde`. The files use KColorScheme's named View, Window, Button, Tooltip, Complementary, Header and Selection sets. Values come from shared semantic tokens and become RGB triples. Every mapped text/background pair is checked at 4.5:1 before output. Unsupported source roles and all mapped values are reported. No palette values are copied from Breeze.

For a native application preview, build the supplied small Qt executable with the **already installed** Qt Widgets and KDE Frameworks development files:

```sh
cmake -S adapters/kde -B dist/kde-preview
cmake --build dist/kde-preview
dist/kde-preview/fjord-kde-preview /absolute/path/to/FjordNight.colors /absolute/path/to/night-preview
```

The program opens its own native test window, captures before/after PNGs and palette metadata, then closes. It sets an application-local palette and `KDE_COLOR_SCHEME_PATH` property; it never writes KDE settings. Use a temporary `XDG_CONFIG_HOME` for a clean baseline. Missing build tools/libraries must not be installed without permission. The preview deliberately leaves native fonts, widget shapes, shadows, motion and disabled/inactive rendering under the installed Breeze style.

The native check produced Night, Day and Aurora previews with readable opaque surfaces, selected text and focused input. The exact application palette and selected text/focus state are captured alongside images. Native rich-text link styling, inactive colour effects, decoration colours and arbitrary applications can deviate from a simple QPalette mapping; raw palette values do not prove every rendered pixel.

## Installation and reversal

For inspection, use the isolated application preview above. It needs no installation or rollback. A future system-wide experiment can copy a uniquely named `.colors` file into `~/.local/share/color-schemes/`, after backing up any same-name file and recording the current scheme. Changing System Settings is a separate explicit user action. Select the previous scheme before removing only the copied file. This task did not change the active global scheme.

## Production decision

Retain this as a **bounded Qt colour proof of concept**, rather than declaring a production desktop adapter. The shared colours transfer; a colour scheme cannot reproduce component layout, fonts, motion, window geometry, all application colour roles or every compositor effect.

Concrete follow-up work before a production adapter:

1. Define the supported Plasma/KDE application and widget-style versions, then test global scheme activation and restoration in a disposable desktop account.
2. Inspect rich-text links, inactive selections, disabled text and native focus decoration; record any style-derived substitutions and actual rendered contrast.
3. Scope Plasma panels and KWin decorations separately, including their opaque-surface/shape limits. GTK and Electron need separate adapter decisions.
4. Choose system-wide installation/version management and retain native screenshot baselines per supported environment.

Sources: the installed Breeze `.colors` format, the installed `KColorScheme` API and KDE's [theme boundaries](https://develop.kde.org/docs/plasma/theme/). Generated colour data and Fjord's preview source use the included MIT licence. Qt/KDE libraries are dynamically linked from the host and are not redistributed in the theme bundle.
