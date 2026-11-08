# Fjord Svelte reference

This workspace contains all 66 component references and the Storybook catalogue. `ThemeScope` defaults to Night and accepts Night, Day, Aurora or the explicit System preference. It scopes `data-fjord-mode`, `data-fjord-preference` and an `identity` string without changing document globals or storing a choice. Import `@fjord/svelte/semantics.css` after the typography/foundation styles for semantic colours and states, including the Fern example identity. System CSS also applies before hydration; the client observes changes and cleans up its listener. Nested scopes remain independent. The source repository documents general token exports and [the accepted local MVP's verification limits](../../docs/verification/xer-120.md).

`Button` and `Badge` require a non-empty localized `label`. `Button` supports primary, secondary, ghost and destructive variants, native button types, disabled and controlled loading. `iconOnly` requires an icon and uses the label as the accessible name. `Badge` is a static status label with an optional decorative icon. See the Components/Button and Badge stories for examples and limitations; component specifications are in the source repository's `docs/components` directory.

`Field`, `Label`, `Input`, `Textarea` and `NativeSelect` compose native, labelled form controls. Spread the control object from Field's child snippet onto its control to connect the ID, help, error, required and disabled states. Use unique IDs and localized text. Input and Textarea support read-only values; NativeSelect provides a single native selection. The Components/Form fields story demonstrates the API and its limits. Import `@fjord/svelte/form-controls.css` after the foundation and semantic styles.

`Checkbox`, `RadioGroup` and `Switch` provide native choice controls. Supply visible localized labels and import `@fjord/svelte/choices.css` after the shared styles. RadioGroup uses a fieldset and one shared form name. The application owns the mixed checkbox state. See Components/Choice controls in Storybook for usage and native validation.

`Card`, `Separator` and `Tabs` provide layout and content navigation. Import `@fjord/svelte/layout-components.css` after the shared styles. Tabs support automatic or manual activation and horizontal or vertical keyboard navigation; inactive panels stay mounted. The Components/Card Separator Tabs story shows the API and its limits.

`Dialog`, `Alert` and `Tooltip` use native modal, live-region and description semantics. Import `@fjord/svelte/overlay-components.css` after the shared styles. Dialog requires a localized title and close label; Tooltip needs one named trigger and supplementary text. The Components/Dialog Alert Tooltip story demonstrates the API and its limits.

`Table`, `Empty`, `Skeleton` and `Spinner` provide data and loading states. Import `@fjord/svelte/data-feedback.css` after the shared styles. Table needs a caption and semantic headers; Skeleton is decorative, and Spinner carries a visible loading label. The Components/Table Empty Skeleton Spinner story shows usage.

`Typography`, `CodeBlock` and `Kbd` provide semantic text, source code and shortcut labels. Import `@fjord/svelte/type-components.css` after the shared styles. CodeBlock validates that highlighted segments reproduce the source, copies the source exactly and accepts localized copy messages. See Components/Typography Code Block Kbd.

The Fjord/Explore story brings mode, identity and density controls together with token inspection, semantic colours, typography and a terminal example. Complete Library contains eleven additional family views and a composed settings/data/messages workspace. Import `@fjord/svelte/complete-components.css` for the added components. The other stories demonstrate component variants and states. Token inspection data is generated from the validated sources before Storybook starts or builds.

Run commands from the repository root:

```sh
mise run reference:check
mise run reference:build
mise run storybook:build
mise run reference:dev
mise run storybook:dev
```

The library builds to `dist/svelte/`, the Vite specimen to `dist/reference/`, and the static catalogue to `dist/storybook/`. Generated files do not belong in version control.

The supporting `Icon` primitive uses the 16 pinned Phosphor Fill SVGs and retains their original MIT notice under `icons/`. Import `@fjord/svelte/foundations.css` for shared layout, density, focus, motion and system-colour rules. Wrap its controls in `.fjord-foundation.fjord-type-body` and select a mode with `data-fjord-mode`. Decorative icons are the default. A meaningful icon requires `decorative={false}` and a non-empty localized `label`; an icon-only button receives its accessible name on the button.

Consumers install the versioned local archive produced by `mise run test:types`. Import `ThemeScope` and the `Mode` type from `@fjord/svelte`. The package requires Svelte 5.57.2 or a compatible later 5.x version. Its own sources use the repository's MIT licence. Dependencies retain their licences.

Import `@fjord/svelte/typography.css` once in the application entry. It includes nine self-hosted WOFF2 faces and the 15 `.fjord-type-*` classes. Keep its relative font directories and original OFL licences when redistributing. Sizes use `rem`. Contextual ligatures default to on; comments and keywords use standard slanted italic with handwritten forms off. Set `data-fjord-ligatures="off"` to disable ligatures, or explicitly opt into handwritten forms with `data-fjord-cursive="on"` on the document root. The archive includes the font manifest, inspection evidence and original notices under `fonts/`.

## Complete-library candidate

The package version is `1.0.0-rc.1`; it is a local release candidate. Existing MVP exports remain available. Import `@fjord/svelte/work.css` or `@fjord/svelte/gaming.css` after semantics, then choose `identity="work"` or `identity="gaming"` on ThemeScope. Return to `identity="fjord"` to reset. All three modes use the same component implementation. These CSS exports are supplied by the built distribution.

The source repository's `docs/components/matrix.json` links every public component to its contract. The new components require localized labels and stable unique IDs where exposed. Applications own persistence, network operations, upload lifecycles, server validation and command registration. The data table sorts and filters resident rows; it does not fetch or virtualize data. The conversation example keeps messages in memory.

See `docs/distribution.md`, `docs/migration-v1.md` and `docs/support-matrix.md` in the complete local archive for installation, compatibility and verification limits. Browser accessibility-tree checks do not establish spoken screen-reader output.
