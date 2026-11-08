<script lang="ts">
  import Calendar from "./Calendar.svelte";
  import Popover from "./Popover.svelte";
  let {
    id,
    label,
    value = $bindable(""),
    name,
    min,
    max,
    locale = "en-GB",
    weekStartsOn = 1,
    error,
    required = false,
    disabled = false,
  }: {
    id: string;
    label: string;
    value?: string;
    name?: string;
    min?: string;
    max?: string;
    locale?: string;
    weekStartsOn?: number;
    error?: string;
    required?: boolean;
    disabled?: boolean;
  } = $props();
  let open = $state(false);
</script>

<div class="fjord-widget fjord-date-picker">
  <label for={id}>{label}</label><input
    {id}
    {name}
    type="date"
    bind:value
    {min}
    {max}
    {required}
    {disabled}
    aria-invalid={error ? "true" : undefined}
    aria-describedby={error ? `${id}-error` : undefined}
  />{#if !disabled}<Popover
      id={`${id}-calendar`}
      label={`Choose ${label}`}
      bind:open
      ><Calendar
        {label}
        bind:value
        {min}
        {max}
        {locale}
        {weekStartsOn}
        onchange={() => (open = false)}
      /></Popover
    >{/if}{#if error}<p id={`${id}-error`}>{error}</p>{/if}
</div>
