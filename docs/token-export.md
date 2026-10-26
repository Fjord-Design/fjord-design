# Token exports

Run `mise run tokens:export` to produce four exports for Night, Day and Aurora in `dist/`. Each export composes the same source layers and validates required roles, aliases and contrast. Output is deterministic and records the contract version, mode and sRGB colour space.

| File                  | Content                                                                                 |
| --------------------- | --------------------------------------------------------------------------------------- |
| `night.dtcg.json`   | DTCG 2025.10 tokens with aliases and metadata.                                          |
| `night.resolved.json` | Flat token paths with type, resolved value, source layer and original value.            |
| `night.tokens.css`    | CSS custom properties derived from every token.                                         |
| `night.wezterm.toml`  | WezTerm colour scheme with foreground, background, cursor, selection and 16 ANSI slots. |

Use `day` or `aurora` in the filename for the other modes. Override a destination and theme without changing source files:

```sh
PYTHONPATH=src python3 -m fjord_tokens export --format css --mode day --theme examples/themes/fern.tokens.json --output dist/fern-day.tokens.css
```

`--override` accepts additional files in order, after the chosen theme. Unknown paths, invalid aliases, missing required roles and insufficient contrast fail the export. The [token contract](token-contract.md) documents precedence and the supported DTCG profile.

## CSS and precision

CSS names start with `--fjord-token-` and append the full token path with dots changed to hyphens. A collision stops export. A typography composite expands into `-font-family`, `-font-size`, `-font-weight`, `-letter-spacing` and `-line-height`; shadows become CSS shadow lists. Dimensions and durations retain their source units and numeric precision. Font-family names are quoted when needed. Colours use sRGB HEX with alpha where required. Conversion rounds each channel to the nearest 8-bit value. Contrast checks run on both the original sRGB values and the emitted HEX fallback. Resolved JSON retains the source values and full numeric precision.

Import one mode's CSS into a page or scope the generated declarations for a custom integration. The Svelte reference also provides its own scoped semantic and foundation styles. General CSS token names are distinct from those reference styles.

## WezTerm

The MVP target is WezTerm `20240203-110809-5046fc22` on macOS. The generated scheme maps all 16 ANSI entries in order and uses the same semantic colours as the web reference. Consult [WezTerm's appearance documentation](https://wezterm.org/config/appearance.html) for supported fields and [scheme loading](https://wezterm.org/config/lua/wezterm.color/load_scheme.html) for the TOML loader.

To try a generated file, preserve your existing configuration and place the TOML beside it with a new name. Merge this snippet into the existing `wezterm.lua`; use the actual filename and retain other settings:

```lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()
local scheme = wezterm.color.load_scheme(wezterm.config_dir .. '/fjord-night.toml')
config.colors = scheme
return config
```

Select a different exported file to change mode. Remove the `config.colors` assignment to restore your previous colour settings, or restore your saved configuration. Do not overwrite an existing file. The scheme does not configure shell prompts, fonts, window styling or automatic appearance switching. Applications that emit their own colours can override these defaults.

The installed WezTerm loader accepted each generated mode through an isolated temporary configuration on macOS 26.7.1. Visual inspection of ANSI colours, the cursor, selections and prompt context remains open until the Mac can be unlocked. This does not establish native visual support yet.
