# Button

A Button triggers an action. It renders a native HTML `button`, with a required non-empty localized `label`. The default type is `button`; choose `submit` or `reset` explicitly inside a form. Supply `icon` for an optional decorative icon. Set `iconOnly` with an icon and a label for a named icon button. Icon-only controls need a recognizable icon and context; use a visible label when that is unclear.

The button contains an optional icon and a label. The label stays visible during loading, except on an icon-only button where it supplies the accessible name. The loading icon replaces the normal icon. Primary uses the component tokens below. Other variants use semantic roles directly:

| Variant | Foreground | Default fill | Hover and pressed |
| --- | --- | --- | --- |
| Primary | `on-accent` | `accent` | `accent-hover`; pressed adds `shadow-pressed` |
| Secondary | `ink` | `bg-surface`, `line-strong` border | `bg-hover` / `bg-active` |
| Ghost | `ink` | Transparent | `bg-hover` / `bg-active` |
| Destructive | `on-danger` | `danger` | Same colours, raised / pressed shadow |

The permitted contrast pairs are checked by the token validator. Place a ghost button on surfaces approved for `ink` in [semantic colours](../semantic-colors.md). Disabled buttons use `ink-disabled` on `bg-surface`, with native `disabled` behaviour. Long text wraps; coarse pointer targets use `target-touch`. Forced-colour mode uses system colours and reduced motion stops the spinner.

## Loading and forms

Set `loading` when the application starts pending work and clear it when that work ends. The component retains focus and normal foreground contrast, sets `aria-busy` and `aria-disabled`, and prevents activation and native submit/reset from its click while pending. The application owns the asynchronous operation and must update `loading` promptly; the component cannot prevent a caller from invoking submission directly through browser APIs. Guard the submit handler if a form can be submitted by other means. Native `disabled` removes the button from normal keyboard navigation; use it for unavailable actions, not for a pending action that needs to retain focus.

```svelte
<script lang="ts">
  import { Button } from '@fjord/svelte';
  let pending = $state(false);
</script>

<Button label="Save" loading={pending} onclick={() => { pending = true; }} />
<Button label="Search" icon="search" iconOnly variant="secondary" />
```

The application clears `pending` after completion and communicates the result. `Button` does not render a link or implement a toggle; use the appropriate native semantics for navigation and persistent choices. Import the foundation and semantic styles described in the [Svelte reference](../../reference/svelte/README.md).

## Primary component tokens

Use `component.button.primary.*` roles when implementing the primary action. Each token aliases a semantic role; theme changes must reach the button through that relationship.

| Use or state | Component token | Semantic role |
| --- | --- | --- |
| Default background | `background` | `semantic.color.accent` |
| Label and icon | `foreground` | `semantic.color.on-accent` |
| Hover background | `hover-background` | `semantic.color.accent-hover` |
| Pressed background | `pressed-background` | `semantic.color.accent-hover` |
| Disabled label | `disabled-foreground` | `semantic.color.ink-disabled` |
| Focus ring | `focus-color` | `semantic.color.focus-ring` |
| Focus width | `focus-width` | `semantic.border-width.border-focus` |
| Shape | `radius` | `semantic.radius.radius-full` |
| Horizontal padding | `padding-inline` | `semantic.spacing.space-4` |
| Vertical padding | `padding-block` | `semantic.spacing.space-2` |
| Label size | `font-size` | `semantic.font-size.text-md` |
| Label font | `font-family` | `semantic.font.sans` |

Keep the focus outline offset from the control so its adjacent colour is the containing surface. Preserve focus during loading. Loading suppresses repeated activation and presents a text or accessible status; disabled controls suppress activation and use the disabled role only for their label. Platform implementations must supply the appropriate native semantics, keyboard activation and focus behaviour.

The validator requires every component binding to reference a semantic role directly. It checks label contrast for default, hover and pressed backgrounds and focus colour against the six normal surfaces after overrides. Disabled text is excluded from contrast thresholds.

Do not put literal colours or primitive palette references in component specifications or implementations. Platform adapters map these roles to their native styling properties. This specification does not prescribe React, CSS or a desktop toolkit.
