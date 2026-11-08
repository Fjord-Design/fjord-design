<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    title,
    message,
    priority = "polite",
    open = $bindable(true),
    children,
    ondismiss,
  }: {
    title: string;
    message: string;
    priority?: "polite" | "assertive";
    open?: boolean;
    children?: Snippet;
    ondismiss?: () => void;
  } = $props();
  let root = $state<HTMLDivElement>();
  let previous: HTMLElement | null = null;
  $effect(() => {
    if (open) previous = document.activeElement as HTMLElement;
  });
  function dismiss() {
    if (
      root?.contains(document.activeElement) &&
      previous?.isConnected &&
      !root.contains(previous)
    )
      previous.focus();
    open = false;
    ondismiss?.();
  }
</script>

<div aria-live={priority} aria-atomic="true">
  {#if open}<div class="fjord-widget fjord-toast" bind:this={root}>
      <strong>{title}</strong>
      <p>{message}</p>
      {#if children}{@render children()}{/if}<button
        type="button"
        aria-label={`Dismiss ${title}`}
        onclick={dismiss}>Dismiss</button
      >
    </div>{/if}
</div>
