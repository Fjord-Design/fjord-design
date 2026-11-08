<script lang="ts">
  import type { Snippet } from "svelte";
  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="fjord-widget fjord-button-group" role="group" aria-label={label}>
  {@render children()}
</div>
