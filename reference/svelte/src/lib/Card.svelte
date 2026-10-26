<script lang="ts">
  import type { Snippet } from "svelte";
  let { as = "div", children }: { as?: "div" | "article"; children: Snippet } =
    $props();
</script>

<svelte:element this={as} class="fjord-card"
  >{@render children()}</svelte:element
>
