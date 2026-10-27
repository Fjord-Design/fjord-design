# Create a Fjord theme

This guide uses the Fern accent variant to create a separate identity. It keeps Fjord's sources and Svelte component code unchanged. Fern is included as a worked example; its aliases change accent and selection roles in Night, Day and Aurora.

## 1. Set up and inspect

From a clean checkout, install the exact tools and dependencies in `mise.toml` and `bun.lock`, then build and inspect the catalogue:

```sh
mise install
mise run setup
mise run ci
mise run storybook:dev
```

Open the URL printed by Storybook. In Fjord/Explore, switch Identity between Fjord and Fern and Mode between Night and Day. The same components render both identities. Selecting Fjord restores the default. System follows the operating system's light or dark preference; Aurora is an explicit mode. Density changes the example layout and is independent of identity.

## 2. Create an identity

Copy `examples/themes/fern.tokens.json` to a separate file in your own project. Give each changed semantic colour a complete token definition, with its type, value and description. Existing semantic colours can be referenced by alias. Keep the `$extensions.org.xerrion.fjord` metadata from the example. The composition order is base, selected mode, theme, then repeated overrides from left to right. See the [token contract](token-contract.md) for allowed paths and values.

Generate an isolated Svelte stylesheet and an inspectable export. These commands use the example directly so they can be run before creating a new file:

```sh
PYTHONPATH=src python3 -m fjord_tokens theme-css --identity my-fern --theme examples/themes/fern.tokens.json --output dist/my-fern.theme.css
PYTHONPATH=src python3 -m fjord_tokens export --mode day --format resolved --theme examples/themes/fern.tokens.json --output dist/my-fern-day.resolved.json
```

The scoped stylesheet covers all three modes and System's light preference. The identity name must contain lower-case letters, digits and hyphens. `theme-css` currently accepts semantic colour overrides only. It rejects changes to layout, typography, shadows or component token definitions so they cannot be silently omitted. The general [token exports](token-export.md) support the wider contract for platform adapters.

Each command validates every base mode, aliases and the selected composition. The scoped stylesheet checks all three modes and fails when a changed colour causes an invalid sRGB or HEX contrast pair. Its selectors apply only to the chosen identity. Keep `tokens/` as the source of truth and store your theme file separately.

## 3. Use it in Svelte

Build the local versioned archive with `mise run test:types`, then install the resulting `dist/fjord-svelte-<version>.tgz` in the consuming application with its package manager. Import the bundled fonts and shared styles once. Serve `my-fern.theme.css` from your application and import it after the shared styles:

```ts
import "@fjord/svelte/typography.css";
import "@fjord/svelte/foundations.css";
import "@fjord/svelte/semantics.css";
import "./my-fern.theme.css";
```

```svelte
<script lang="ts">
  import { Button, ThemeScope } from "@fjord/svelte";
</script>

<ThemeScope mode="day" identity="my-fern">
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <Button label="Save" />
  </main>
</ThemeScope>
```

Use `identity="fjord"` to return to the standard theme. Use `mode="night"`, `"day"`, `"aurora"` or `"system"`. `ThemeScope` does not persist preferences; the application owns storage and user settings. Add the component stylesheets documented in the [Svelte reference](../reference/svelte/README.md) for the components you use. Fern changes the interface accent, not the fixed brand SVG fills. Verify the logo against its background using the [brand instructions](../assets/brand/README.md).

## 4. Export for another platform

An adapter maps the resolved semantic roles to names supported by its target. It may select or combine supported values, but it must document any lost states, alpha, typography, motion or colour precision. It must not silently change source tokens or claim that an unsupported platform feature works. The WezTerm example and its limitations are in the [export guide](token-export.md).

Run `mise run check` and regenerate each target after changes. Contrast and alias validation are necessary for every mode; native target testing is still needed for rendering, interaction and assistive technology. Review [version and token lifecycle rules](versioning.md) before updating a consumer. Keep each adapter and its target version alongside its mapping and tests.
