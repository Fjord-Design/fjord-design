# Fjord for daily WezTerm use

Build with `mise run adapters:wezterm`. Copy the generated directory to a new `fjord/` folder beside your existing WezTerm configuration, keeping `night.toml`, `day.toml`, `aurora.toml`, `init.lua` and the MIT licence. Back up any existing same-name directory and your configuration first. Generation does not install anything.

Add only these lines to your existing configuration after constructing `config` and before returning it:

```lua
local fjord = dofile(wezterm.config_dir .. '/fjord/init.lua')
fjord.apply(config, 'night')
```

Use `day` or `aurora` to change mode. The helper loads and validates the scheme before merging its colours, preserving other colour fields and every unrelated configuration key. It does not choose fonts, a shell, key bindings, window geometry, opacity or motion. If your configuration uses a different directory, replace the path explicitly and pass that folder as the third argument to `fjord.apply`. Avoid competing later `config.colors` assignments.

For an upgrade, generate into another unused folder, review the report, load it in an isolated `--config-file` test, then deliberately replace the installed folder. Keep the previous folder and configuration backup. To roll back, restore those Fjord files and the prior activation line. To uninstall, remove only the activation lines/Fjord folder and restore your previous colour selection; retain all other settings and later edits.

The original M0 target is WezTerm 20240203-110809-5046fc22 on macOS. Native Linux coverage is separately recorded for WezTerm 20260716-195552-76b606ec. Other versions, arbitrary terminal programs and macOS visual equivalence are not implied. See the central support matrix and verification record for exact checks.

`fixtures.sh` covers explicit normal/bright ANSI slots, truecolour, OSC 8 links, diff, errors, cursor/selection and prompt context. A colour scheme cannot recolour application-supplied truecolour or enforce arbitrary foreground/background combinations. ANSI black is a background role in Night/Aurora. Use explicit bright-black for readable hints; SGR dim and implicit bold-to-bright behaviour are outside the palette guarantee. The theme does not change these runtime preferences.

The optional standalone `prompt.py` reads `prompt-colors.json`, prints labelled Git status, branch, environment and duration, and requires only Python plus Git for repository context. Run it manually with `python3 /path/to/fjord/prompt.py --mode night --environment dev --duration-ms 12`; `--plain` removes colour while retaining every status label. It never edits shell startup files or launches a prompt engine. Shell integration and measuring a real command's duration remain application/shell responsibilities. Base colour use does not require this helper.

Use already installed Cascadia Code or Fira Code if desired; ligatures and italic depend on font and terminal configuration. No font files are included here. Theme colours, helpers and fixtures use the included MIT licence.
