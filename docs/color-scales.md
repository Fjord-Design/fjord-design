# Colour scales and conversion

Run `mise run palette` to build the offline colour specimen at `dist/palette.html` and its machine-readable evidence at `dist/palette.json`. Both consume the approved token sources and the same checks as token builds.

## Source and scales

Canonical UI values are DTCG `srgb` colours with three encoded components in `[0,1]` and explicit alpha. The seven OKLCH references remain exact inspiration metadata in `designSeeds` in the base source. The first reference's stray trailing hyphen in the brief is punctuation; it is absent from the stored colour expression. The parser rejects it as part of a colour.

| Original reference | sRGB HEX | Related Night role |
| --- | --- | --- |
| `oklch(32.9478% 0.034021 247.264)` | `#273746` | `bg-hover` |
| `oklch(20.8843% 0.018341 245.245)` | `#111920` | `bg-base` |
| `oklch(34.9975% 0.034292 250.45)` | `#2d3c4c` | `bg-active` |
| `oklch(76.0906% 0.065221 71.267)` | `#ccab84` | `latte` |
| `oklch(86.9933% 0.028271 46.055)` | `#e5cfc5` | `foam` |
| `oklch(75.9325% 0.065969 10.27)` | `#d7a0a7` | `rose` |
| `oklch(86.9339% 0.02788 129.841)` | `#ced8c5` | `sage` |

All seven references fit in sRGB. Their rounded HEX values reproduce the related Night colours. Day and Aurora use the already approved role values. Each semantic role retains its name across modes. Colour scales are named by purpose, rather than arbitrary numbered stops:

| Scale | Roles and use |
| --- | --- |
| Surfaces | `bg-sunken`, `bg-base`, `bg-surface`, `bg-raised`; `bg-hover` and `bg-active` are interaction fills |
| Text and boundaries | `ink`, `ink-muted`, `ink-subtle`, `ink-disabled`; `line`, `line-strong` and `focus-ring` separate or identify controls |
| Interaction | `accent`, `accent-hover`, `on-accent`, `accent-soft`, `accent-ink`, `selection` |
| Accents | `frost`, `aurora`, `lilac`, `sage`, `fern`, `latte`, `foam`, `rose` |
| Status | `info`, `success`, `warning`, `danger`, their `*-soft` fills and `on-danger` |

These 36 specimen roles cover the main scales. The JSON evidence includes all 81 semantic colour roles per mode, plus every required contrast pair. Normal surfaces get lighter as elevation rises in Night and Aurora. Day uses pale surfaces and darker hover/pressed fills. Text prominence runs from `ink` through muted/subtle to disabled; disabled text has no normal-text contrast claim.

Night uses frost-blue interaction, Day deep blue and Aurora teal. Rose remains secondary. Warm latte, foam and rose accents occupy small details and syntax, while large surfaces stay cool. ANSI yellow uses latte; its bright slot uses the explicitly defined café tone. Sage is a decorative accent; success uses fern. Status also needs text or a symbol. These values preserve the M0 palette and the existing [platform mappings](platforms.md).

## Conversion and gamut

The converter in `src/fjord_tokens/color.py` uses the published [OKLab linear-sRGB matrices](https://bottosson.github.io/posts/oklab/) and the sRGB transfer function. OKLCH becomes OKLab through `a = C cos(h)` and `b = C sin(h)`, then linear RGB, then encoded sRGB. The coordinate definitions are described in [CSS Color 4](https://www.w3.org/TR/css-color-4/#ok-lab). Fjord's source colour shape follows the [DTCG Color Module](https://www.designtokens.org/tr/2025.10/color/).

The seed parser accepts Fjord's `oklch(L% C H)` subset: finite lightness from 0 to 100 percent, chroma from 0 to 1, and numeric hue in degrees. Hue wraps modulo 360. It accepts no alpha, `none` channels or additional CSS syntax. Internal tuple conversion uses lightness from 0 to 1. Approved UI sources remain sRGB; the broader CSS colour syntax is outside this parser's contract.

Fjord's fallback holds lightness and hue constant and reduces chroma to the sRGB gamut boundary. For an out-of-gamut colour it performs exactly 32 binary-search iterations between zero and the requested chroma. The gamut tolerance is `1e-7` in encoded sRGB; only that numerical boundary residue is clamped to `[0,1]`. Lightness endpoints produce black or white with zero chroma. This is an explicit Fjord mapping policy, independent of a browser's own gamut-mapping choice.

For example, `oklch(70% 0.4 30)` is outside sRGB. The converter reduces chroma to about `0.1915123605` and produces `#ff6551`. The report records whether a reference was mapped and its resulting chroma. Inputs outside the parser's finite bounds fail before output is written.

Each HEX channel is `floor(component * 255 + 0.5)`, serialized in lowercase with two digits. Opaque colours use six digits; alpha adds two digits when needed. Optional DTCG `hex` hints must match the rounded RGB components. OKLCH coordinates displayed in the specimen are inspection labels; their display rounding does not feed the source or contrast calculation.

## Contrast evidence

Every required pair is checked twice: once on full-precision source sRGB, then on the exact components recovered from its final HEX value. Ratios are compared without rounding against the existing thresholds: 4.5:1 for text, 3:1 for specified control boundaries/focus, 5:1 for code comments and 7.5:1 for comments on Night's base surface. The [token contract](token-contract.md) defines the complete pair policy and exceptions.

There are 143 pairs in Night, 144 in Day and 143 in Aurora, including semantic usage and component bindings. The palette report records both ratios and each minimum. It validates all modes before writing. A passing source ratio cannot mask a failing rounded fallback. Alpha remains prohibited in checked pairs; scrims and shadow layers retain their separate contracts.

The specimen draws HEX swatches and gives each demonstration label whichever of black or white has higher contrast. That sample label does not approve a colour for other text or surface combinations. Open the contrast disclosure for the actual allowed pairs. Original references are inspiration metadata, so their presence alone gives no UI contrast guarantee.

For theme and override inspection:

```sh
mise exec -- python3 -m fjord_tokens palette --format json \
  --theme examples/themes/soft-windows.tokens.json \
  --override examples/overrides/compact-spacing.tokens.json \
  --output dist/palette-variant.json
```

The layer order and source protections match the token build contract. The palette JSON is a verification report; general CSS and resolved-token export are delivered by XER-116.
