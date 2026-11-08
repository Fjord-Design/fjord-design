<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    label,
    children,
    prefix,
    suffix,
    error,
  }: {
    label: string;
    children: Snippet;
    prefix?: string;
    suffix?: string;
    error?: string;
  } = $props();
  const uid = $props.id();
</script>

<div
  class="fjord-widget fjord-input-group"
  role="group"
  aria-label={label}
  aria-describedby={error ? `${uid}-error` : undefined}
>
  {#if prefix}<span>{prefix}</span>{/if}{@render children()}{#if suffix}<span
      >{suffix}</span
    >{/if}
</div>
{#if error}<p class="fjord-group-error" id={`${uid}-error`}>{error}</p>{/if}
