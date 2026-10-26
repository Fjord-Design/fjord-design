# Form fields

Use Field to group one control with a visible Label, optional help and optional error. Give each Field a unique, stable `id` and a non-empty localized `label`. Its child snippet receives a control object; spread that object onto one Input, Textarea or NativeSelect. It supplies the control ID, required and disabled states, and references to help and error text. An error sets `aria-invalid` and is added to `aria-describedby` alongside help. Applications decide when to validate, provide localized messages and manage focus or announcements after errors.

Import `@fjord/svelte/form-controls.css` after the shared foundation and semantic styles to style the controls.

```svelte
<script lang="ts">
  import { Field, Input } from '@fjord/svelte';
  let email = $state('');
  let error = $state('');
</script>

<Field id="email" label="Email" required help="We use this to reply." {error}>
  {#snippet children(control)}
    <Input {...control} type="email" name="email" autocomplete="email" bind:value={email} />
  {/snippet}
</Field>
```

The required marker is visible; the native `required` attribute communicates the state. The application must keep the control ID and its derived `-help` and `-error` IDs unique across the page, and spread the received properties without overriding them accidentally. Label can also be used separately with its `for` value pointing to the control's ID. A control used without Field still needs a visible label or another accessible name.

## Controls and states

| Component | Native element | Behaviour |
| --- | --- | --- |
| Input | `input` | Text, email, password, search, telephone and URL types; forwards native attributes such as `name`, `autocomplete`, `readonly` and `required`. |
| Textarea | `textarea` | Multiline input; vertically resizable and supports `rows`, `name`, `autocomplete` and `readonly`. |
| NativeSelect | `select` | Single selection using native options, keyboard handling and platform menu. Its children are `option` and `optgroup` elements. |

Each value can be bound with `bind:value`. Native disabled controls are not editable, focusable or submitted; their labels and help remain readable. Read-only Input and Textarea remain focusable and are submitted. HTML select has no read-only state; use static text when a choice must be displayed without editing. Native Select does not provide custom search or a multiple-selection contract. Placeholder text supplements a label and is not a substitute for it.

The default control uses `bg-surface`, `ink` and `line-strong`. Read-only uses `bg-sunken`; focus uses `focus-ring`, `border-focus` and `focus-offset`. Error text and invalid borders use `danger`. Disabled control text uses `ink-disabled` and its help uses `ink-muted`. Spacing, size and shape use `space-1`, `space-2`, `space-3`, `control-comfortable`, `target-touch` and `radius-control`. Colours follow the semantic stylesheet in Night, Day, Aurora and custom identities. Forced-colour mode uses system colours.

The example in Storybook, Components/Form fields, checks validity on submit and displays an error without sending data. Browser validation and `autocomplete` are native browser features; saved autofill suggestions depend on the user's browser and settings. The application remains responsible for validating submitted data.
