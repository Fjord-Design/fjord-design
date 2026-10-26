<script lang="ts">
  import type { Snippet } from "svelte";
  let { for: target, children }: { for: string; children: Snippet } = $props();
</script>

<label for={target} class="fjord-label">{@render children()}</label>

<style>
  .fjord-label {
    display: block;
    color: var(--fjord-ink);
    font-family: var(--fjord-button-primary-font-family);
    font-size: var(--fjord-button-primary-font-size);
    font-weight: 600;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }
</style>
