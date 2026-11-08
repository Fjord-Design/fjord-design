<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    children,
    outgoing = false,
  }: { children: Snippet; outgoing?: boolean } = $props();
</script>

<div class="fjord-bubble" data-outgoing={outgoing}>{@render children()}</div>
