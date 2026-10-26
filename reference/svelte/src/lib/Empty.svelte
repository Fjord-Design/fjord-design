<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons.generated.js";
  let {
    title,
    description,
    icon = "file",
    children,
  }: {
    title: string;
    description: string;
    icon?: IconName;
    children?: Snippet;
  } = $props();
  const checkedTitle = $derived.by(() => {
    if (!title?.trim() || !description?.trim())
      throw new Error("Empty requires a title and description.");
    return title;
  });
</script>

<section class="fjord-empty">
  <Icon name={icon} size="empty" />
  <h3>{checkedTitle}</h3>
  <p>{description}</p>
  {#if children}<div class="fjord-empty-action">{@render children()}</div>{/if}
</section>
