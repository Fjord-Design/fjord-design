<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  type Props = Omit<
    HTMLInputAttributes,
    "type" | "checked" | "indeterminate" | "class" | "children" | "aria-label"
  > & {
    label: string;
    checked?: boolean;
    indeterminate?: boolean;
  };
  let {
    label,
    checked = $bindable(false),
    indeterminate = $bindable(false),
    disabled = false,
    required = false,
    ...rest
  }: Props = $props();
  const checkedLabel = $derived.by(() => {
    if (!label?.trim()) throw new Error("Checkbox requires a non-empty label.");
    return label;
  });
</script>

<label class="fjord-choice">
  <input
    {...rest}
    type="checkbox"
    class="fjord-choice-input"
    {disabled}
    {required}
    bind:checked
    bind:indeterminate
  />
  <span class="fjord-choice-label"
    >{checkedLabel}{#if required}<span aria-hidden="true"> *</span>{/if}</span
  >
</label>
