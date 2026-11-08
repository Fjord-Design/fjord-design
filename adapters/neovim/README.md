# Fjord for Neovim

Generate with `mise run adapters:neovim`. Add the resulting `dist/adapters/neovim/` directory to Neovim's runtime path, or copy its `colors/` directory into a new, unused package directory such as `~/.local/share/nvim/site/pack/fjord/start/fjord/`. Keep its MIT licence. Do not replace an existing directory without a backup.

Choose `:colorscheme fjord-night`, `:colorscheme fjord-day` or `:colorscheme fjord-aurora`. `:colorscheme fjord` follows your current `background` preference. To persist a choice, add only `vim.cmd.colorscheme("fjord-night")` to your existing configuration after making a backup. No installer edits `init.lua`, shell configuration or plugin registrations.

Upgrade by building into a new directory, inspecting `report.json`, testing it and then replacing only the Fjord package. Restore the previous package and colourscheme to roll back. Uninstall by selecting your prior colourscheme, removing only the Fjord package and removing your own activation line. The scheme sets `termguicolors` and `background` for the current session; restore their prior values if needed.

The target is Neovim 0.12.6 in Linux WezTerm 20260716-195552-76b606ec. **Generation is tested; Neovim runtime and terminal appearance are not tested.** The owner chose generation without installing the missing Neovim runtime on 8 November 2026. No plugin, OS or other Neovim version support is claimed.

The mapping covers editor floats/status/tabs/completion, conventional syntax groups, Treesitter captures, LSP diagnostics/references, selection/search, diff, cursor and 16 ANSI slots. Foreground overrides on selection/search/diff preserve the shared highlighted-text contract. Every colour comes from the semantic source. Opaque Normal/NormalFloat backgrounds reject transparency. All emitted documented text and indicator pairs are checked; user overrides, terminal SGR dim/truecolour and renderer/font changes need native checks.

Comments and keywords request ordinary italic. Fonts, ligatures, cursor shape, key bindings, terminal opacity and motion remain terminal/user configuration. Diagnostic underline styles request curl/dash/solid/dot; terminals may fall back to plain underline. No external plugin is required. `netrw` file-browser groups are provided, but its native integration is not verified; no third-party plugin integration is claimed.

For a disposable check on a machine with the target runtime, build and run `mise run test:neovim`. The test uses `--headless --clean`, an isolated data/config environment and built-in diagnostic APIs. Then inspect the bundled JSON/shell/TypeScript fixtures and native diff in the terminal for every mode; record exact application/OS/font versions and native screenshots before claiming visual support. It does not install language servers or Treesitter parsers.

Custom identities use `--theme path` and repeated `--override path`; `--name fjord-work` gives separately installable names. Source, mappings and MIT licence belong to Fjord. Primary contracts: [highlight API](https://neovim.io/doc/user/api.html#nvim_set_hl()), [highlight groups](https://neovim.io/doc/user/syntax.html#highlight-groups), [Treesitter captures](https://neovim.io/doc/user/treesitter.html#treesitter-highlight) and [diagnostics](https://neovim.io/doc/user/diagnostic.html).
