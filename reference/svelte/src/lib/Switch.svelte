<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  type Props = Omit<
    HTMLInputAttributes,
    | "type"
    | "checked"
    | "indeterminate"
    | "class"
    | "children"
    | "role"
    | "aria-label"
  > & { label: string; checked?: boolean };
  let {
    label,
    checked = $bindable(false),
    disabled = false,
    required = false,
    ...rest
  }: Props = $props();
  const checkedLabel = $derived.by(() => {
    if (!label?.trim()) throw new Error("Switch requires a non-empty label.");
    return label;
  });
</script>

<label class="fjord-choice">
  <input
    {...rest}
    type="checkbox"
    role="switch"
    class="fjord-switch-input"
    {disabled}
    {required}
    bind:checked
  />
  <span class="fjord-choice-label"
    >{checkedLabel}{#if required}<span aria-hidden="true"> *</span>{/if}</span
  >
</label>
