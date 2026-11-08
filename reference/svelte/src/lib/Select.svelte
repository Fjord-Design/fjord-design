<script lang="ts">
  import Combobox from "./Combobox.svelte";
  import type { SelectOption } from "./complete-types.js";
  let {
    id,
    label,
    options,
    value = $bindable(""),
    name,
    disabled = false,
    required = false,
    loading = false,
    error,
  }: {
    id: string;
    label: string;
    options: SelectOption[];
    value?: string;
    name?: string;
    disabled?: boolean;
    required?: boolean;
    loading?: boolean;
    error?: string;
  } = $props();
</script>

<Combobox
  {id}
  {label}
  {options}
  bind:value
  {name}
  {disabled}
  {required}
  {loading}
  {error}
  editable={false}
/>
