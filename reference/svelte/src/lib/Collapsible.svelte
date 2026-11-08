<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    label,
    open = $bindable(false),
    children,
  }: { label: string; open?: boolean; children: Snippet } = $props();
</script>

<details class="fjord-widget fjord-collapsible" bind:open>
  <summary>{label}</summary>
  <div>{@render children()}</div>
</details>
