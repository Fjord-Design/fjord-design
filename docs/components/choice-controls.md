# Choice controls

Checkbox, RadioGroup and Switch use native inputs. Each needs a visible, non-empty localized label. Import `@fjord/svelte/choices.css` after the shared foundation and semantic styles.

| Component | Meaning and anatomy | Form behaviour |
| --- | --- | --- |
| Checkbox | A square control and its label represent an independent choice. `checked` is binary; `indeterminate` represents a partial selection. | A named, checked input submits its value. A mixed state does not change what is submitted; `checked` determines submission. |
| RadioGroup | A fieldset and legend contain labelled options with one shared name. Each option has a unique non-empty value and label. | At most one enabled selection is submitted under the group name. `required` uses native validation. |
| Switch | A track, movable marker and stable label represent an on/off setting. | It is a checkbox with native switch semantics. A named, checked switch submits its value. |

Bind to `checked`, `indeterminate` or RadioGroup `value` when the application owns state. A click or Space changes a checkbox or switch; native radio groups support Space and arrow keys. Disabled controls are skipped by normal keyboard navigation and omitted from submitted form data. A required switch or checkbox must be checked to submit. The application calculates a mixed checkbox from its child choices and must update that state as those choices change. HTML does not serialize the indeterminate property, so it becomes visible after the Svelte client hydrates.

```svelte
<script lang="ts">
  import { Checkbox, RadioGroup, Switch } from '@fjord/svelte';
  let enabled = $state(false);
  let mixed = $state(true);
  let checked = $state(false);
  let interval = $state<string | undefined>();
  const options = [{ value: 'day', label: 'Daily' }, { value: 'week', label: 'Weekly' }];
</script>

<Checkbox label="Select all" bind:checked bind:indeterminate={mixed} />
<RadioGroup id="interval" name="interval" label="How often?" {options} required bind:value={interval} />
<Switch label="Enable notifications" name="notifications" value="on" bind:checked={enabled} />
```

RadioGroup help and error text are associated with the group and its choices. The application supplies localized errors and manages focus or announcements after validation. IDs derived from the group ID must be unique on the page. `disabled` on the group disables all its choices; individual options can also be disabled. Switch labels describe the setting consistently in both states. Use a checkbox when the partial state has meaning; Switch is binary.

Controls use `bg-surface`, `line-strong`, `ink`, `accent`, `on-accent` and `ink-disabled`. Error borders use `danger`; focus uses `focus-ring`, `border-focus` and `focus-offset`. Sizing uses `icon-default`, `control-comfortable` and `target-touch`; shape uses `radius-control` and `radius-full`. Labels remain readable when a control is disabled. Selected marks, mixed marks and switch position remain visible in forced colours. State changes do not depend on animation.

The Components/Choice controls story demonstrates native validation and form values without sending data. Native keyboard and assistive technology behaviour depends on the platform; the complete release matrix remains part of XER-119.
