<script lang="ts">
  import type { SelectOption } from "./complete-types.js";
  let {
    label,
    options,
    value = $bindable<string[]>([]),
    multiple = false,
    disabled = false,
  }: {
    label: string;
    options: SelectOption[];
    value?: string[];
    multiple?: boolean;
    disabled?: boolean;
  } = $props();
</script>

<div class="fjord-widget fjord-toggle-group" role="group" aria-label={label}>
  {#each options as option}<button
      type="button"
      aria-pressed={value.includes(option.value)}
      disabled={disabled || option.disabled}
      onclick={() => {
        value = value.includes(option.value)
          ? value.filter((v) => v !== option.value)
          : multiple
            ? [...value, option.value]
            : [option.value];
      }}>{option.label}</button
    >{/each}
</div>
