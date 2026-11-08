<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    direction = "ltr",
    children,
  }: { direction?: "ltr" | "rtl"; children: Snippet } = $props();
</script>

<div dir={direction} class="fjord-direction">{@render children()}</div>
