<script lang="ts">
  import {
    iconPaths,
    iconSizes,
    type IconName,
    type IconSize,
  } from "./icons.generated.js";

  type Props = { name: IconName; size?: IconSize } & (
    { decorative?: true; label?: never } | { decorative: false; label: string }
  );

  let { name, size = "default", decorative = true, label }: Props = $props();
  const accessibleLabel = $derived.by(() => {
    if (!decorative && !label?.trim())
      throw new Error("A meaningful icon requires a non-empty label.");
    return decorative ? undefined : label;
  });
</script>

<svg
  class="fjord-icon"
  viewBox="0 0 256 256"
  fill="currentColor"
  focusable="false"
  role={decorative ? undefined : "img"}
  aria-hidden={decorative ? "true" : undefined}
  aria-label={accessibleLabel}
  style:width={`var(--fjord-${"icon-" + size}, ${iconSizes[size] / 16}rem)`}
  style:height={`var(--fjord-${"icon-" + size}, ${iconSizes[size] / 16}rem)`}
>
  {@html iconPaths[name]}
</svg>

<style>
  svg {
    display: inline-block;
    flex-shrink: 0;
    vertical-align: -0.125em;
  }
</style>
