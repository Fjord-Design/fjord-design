<script lang="ts">
  import type { Snippet } from "svelte";
  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<span class="fjord-widget fjord-marker"
  >{@render children()}<span class="fjord-marker-indicator"
    ><span aria-hidden="true">● </span>{label}</span
  ></span
>
