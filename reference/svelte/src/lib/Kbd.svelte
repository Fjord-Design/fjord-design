<script lang="ts">
  import type { Snippet } from "svelte";
  let { children }: { children: Snippet } = $props();
</script>

<kbd class="fjord-kbd fjord-type-code">{@render children()}</kbd>
