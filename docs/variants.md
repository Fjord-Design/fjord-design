# Work and Gaming identities

Work and Gaming are optional semantic-colour overrides of Fjord. They retain the cool, opaque surfaces, readable text, three modes, public token names and component bindings. Work uses a restrained frost accent and information-tinted selection. Gaming uses aurora accents and success/diff-tinted selection. Neither adds animation or changes density, fonts, component behaviour or shell settings.

Sources are [work.tokens.json](../examples/themes/work.tokens.json) and [gaming.tokens.json](../examples/themes/gaming.tokens.json). Composition remains base → mode → theme → ordered overrides. `mise run variants:build` validates every composition and writes scoped CSS, five separately named native bundles, VS Code VSIX files and coverage reports under `dist/variants/`.

For a built Svelte consumer, import the needed stylesheet after semantics:

```ts
import "@fjord/svelte/semantics.css";
import "@fjord/svelte/work.css";
import "@fjord/svelte/gaming.css";
```

Choose `identity="work"` or `identity="gaming"` on ThemeScope. Choose `identity="fjord"` to reset. Mode remains independently Night, Day, Aurora or System. The application owns persistence; ThemeScope does not write settings. The catalogue exposes the same choices for all families, code and terminal examples.

Native bundles live in `dist/variants/<identity>/<target>/`. Follow that target's README using this directory in place of the base bundle. Zed, VS Code, Neovim and KDE metadata use distinct identity names. WezTerm schemes retain `night.toml`, `day.toml`, `aurora.toml` inside the separate directory; pass that directory to `fjord.apply`. Keep the base files/configuration to reset. Never overwrite the user's previous settings as an installation shortcut.

All five generators support these identities at the mapping level. Native Work/Gaming evidence currently covers WezTerm and the bounded KDE/Qt preview. New Zed/VS Code variants and all Neovim variants remain natively unverified; see the [support matrix](support-matrix.md).

Custom web identities may override existing semantic colours only through `theme-css`. They cannot add paths or silently redefine component/layout/type tokens. General token exports permit only values allowed by the shared contract. Invalid aliases, missing modes, non-opaque colours and failed source/HEX contrast stop generation. See the [theme guide](theme-guide.md) and [token contract](token-contract.md).
