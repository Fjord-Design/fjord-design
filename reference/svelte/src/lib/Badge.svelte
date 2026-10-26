<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons.generated.js";
  type Variant = "neutral" | "info" | "success" | "warning" | "danger";
  let {
    label,
    variant = "neutral",
    icon,
  }: { label: string; variant?: Variant; icon?: IconName } = $props();
  const checkedLabel = $derived.by(() => {
    if (!label?.trim()) throw new Error("Badge requires a non-empty label.");
    return label;
  });
</script>

<span class="fjord-badge" data-variant={variant}>
  {#if icon}<Icon name={icon} size="control" />{/if}<span>{checkedLabel}</span>
</span>

<style>
  .fjord-badge {
    box-sizing: border-box;
    display: inline-flex;
    align-items: center;
    gap: var(--fjord-space-1);
    max-inline-size: 100%;
    min-inline-size: 0;
    padding: var(--fjord-space-1) var(--fjord-space-2);
    border-radius: var(--fjord-radius-control);
    font-family: var(--fjord-button-primary-font-family);
    font-size: var(--fjord-button-primary-font-size);
    line-height: 1.4;
    overflow-wrap: anywhere;
    color: var(--fjord-ink);
    background: var(--fjord-bg-surface);
  }
  .fjord-badge > span {
    min-inline-size: 0;
  }
  .fjord-badge[data-variant="info"] {
    color: var(--fjord-info);
    background: var(--fjord-info-soft);
  }
  .fjord-badge[data-variant="success"] {
    color: var(--fjord-success);
    background: var(--fjord-success-soft);
  }
  .fjord-badge[data-variant="warning"] {
    color: var(--fjord-warning);
    background: var(--fjord-warning-soft);
  }
  .fjord-badge[data-variant="danger"] {
    color: var(--fjord-danger);
    background: var(--fjord-danger-soft);
  }
  @media (forced-colors: active) {
    .fjord-badge {
      color: CanvasText;
      background: Canvas;
      border: 1px solid CanvasText;
    }
  }
</style>
