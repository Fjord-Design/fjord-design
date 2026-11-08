<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    label,
    blockSize = 320,
    children,
  }: { label: string; blockSize?: number; children: Snippet } = $props();
  const size = $derived.by(() => {
    if (!Number.isFinite(blockSize) || blockSize <= 0)
      throw new Error("ScrollArea needs a positive finite block size.");
    return blockSize;
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex (Named scroll regions need keyboard scrolling.) -->
<div
  class="fjord-widget fjord-scroll-area"
  role="region"
  aria-label={label}
  tabindex="0"
  style:max-block-size={`${size}px`}
>
  {@render children()}
</div>
