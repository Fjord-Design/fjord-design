<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    ratio = 16 / 9,
    label,
    children,
  }: { ratio?: number; label?: string; children?: Snippet } = $props();
  const checked = $derived.by(() => {
    if (!Number.isFinite(ratio) || ratio <= 0)
      throw new Error("AspectRatio needs a positive finite ratio.");
    return ratio;
  });
</script>

<div
  class="fjord-aspect-ratio"
  style:aspect-ratio={checked}
  role={label ? "img" : undefined}
  aria-label={label}
>
  {#if children}{@render children()}{:else}<span>Media unavailable</span>{/if}
</div>
