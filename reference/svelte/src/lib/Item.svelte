<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    title,
    description,
    metadata,
    href,
    onclick,
    disabled = false,
    current = false,
    icon,
    actions,
  }: {
    title: string;
    description?: string;
    metadata?: string;
    href?: string;
    onclick?: () => void;
    disabled?: boolean;
    current?: boolean;
    icon?: Snippet;
    actions?: Snippet;
  } = $props();
  const valid = $derived.by(() => {
    if ((href || onclick) && actions)
      throw new Error("Interactive Item cannot contain nested actions.");
    return title;
  });
</script>

{#snippet content()}{#if icon}<span aria-hidden="true">{@render icon()}</span
    >{/if}<span class="fjord-item-copy"
    ><strong>{valid}</strong>{#if description}<span>{description}</span
      >{/if}{#if metadata}<small>{metadata}</small>{/if}</span
  >{/snippet}
{#if href}<a
    class="fjord-widget fjord-item"
    {href}
    aria-current={current ? "page" : undefined}
    aria-disabled={disabled || undefined}
    onclick={(e) => {
      if (disabled) e.preventDefault();
    }}>{@render content()}</a
  >
{:else if onclick}<button
    type="button"
    class="fjord-widget fjord-item"
    {onclick}
    {disabled}>{@render content()}</button
  >
{:else}<div class="fjord-widget fjord-item">
    {@render content()}{#if actions}<div class="fjord-item-actions">
        {@render actions()}
      </div>{/if}
  </div>{/if}
