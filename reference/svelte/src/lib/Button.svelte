<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons.generated.js";

  type Variant = "primary" | "secondary" | "ghost" | "destructive";
  type Props = Omit<
    HTMLButtonAttributes,
    "children" | "class" | "aria-label"
  > & {
    label: string;
    variant?: Variant;
    loading?: boolean;
  } & (
      { iconOnly: true; icon: IconName } | { iconOnly?: false; icon?: IconName }
    );

  let {
    label,
    variant = "primary",
    loading = false,
    iconOnly = false,
    icon,
    type = "button",
    disabled = false,
    onclick,
    ...rest
  }: Props = $props();
  const checkedLabel = $derived.by(() => {
    if (!label?.trim()) throw new Error("Button requires a non-empty label.");
    if (iconOnly && !icon)
      throw new Error("An icon-only button requires an icon.");
    return label;
  });

  function activate(
    event: MouseEvent & { currentTarget: EventTarget & HTMLButtonElement },
  ) {
    if (loading) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }
    onclick?.(event);
  }
</script>

<button
  {...rest}
  {type}
  {disabled}
  class="fjord-button"
  data-variant={variant}
  data-icon-only={iconOnly}
  aria-label={iconOnly ? checkedLabel : undefined}
  aria-disabled={loading && !disabled ? "true" : undefined}
  aria-busy={loading ? "true" : undefined}
  onclick={activate}
>
  {#if loading}<span class="spinner"
      ><Icon name="loading" size="control" /></span
    >
  {:else if icon}<Icon name={icon} size="control" />{/if}
  {#if !iconOnly}<span>{checkedLabel}</span>{/if}
</button>

<style>
  .fjord-button {
    box-sizing: border-box;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--fjord-space-2);
    min-block-size: var(--fjord-control-comfortable);
    min-inline-size: var(--fjord-control-comfortable);
    max-inline-size: 100%;
    padding: var(--fjord-button-primary-padding-block)
      var(--fjord-button-primary-padding-inline);
    border: var(--fjord-border-hairline) solid var(--fjord-line-strong);
    border-radius: var(--fjord-button-primary-radius);
    font-family: var(--fjord-button-primary-font-family);
    font-size: var(--fjord-button-primary-font-size);
    line-height: 1.5;
    text-align: center;
    overflow-wrap: anywhere;
    cursor: pointer;
    background: var(--fjord-bg-surface);
    color: var(--fjord-ink);
  }
  .fjord-button[data-icon-only="true"] {
    padding: var(--fjord-space-2);
  }
  .fjord-button[data-variant="primary"] {
    background: var(--fjord-button-primary-background);
    color: var(--fjord-button-primary-foreground);
    border-color: var(--fjord-button-primary-background);
  }
  .fjord-button[data-variant="ghost"] {
    background: transparent;
    border-color: transparent;
  }
  .fjord-button[data-variant="destructive"] {
    background: var(--fjord-danger);
    color: var(--fjord-on-danger);
    border-color: var(--fjord-danger);
  }
  .fjord-button:not(:disabled):not([aria-disabled="true"]):hover {
    background: var(--fjord-bg-hover);
  }
  .fjord-button[data-variant="primary"]:not(:disabled):not(
      [aria-disabled="true"]
    ):hover {
    background: var(--fjord-button-primary-hover-background);
    border-color: var(--fjord-button-primary-hover-background);
  }
  .fjord-button[data-variant="destructive"]:not(:disabled):not(
      [aria-disabled="true"]
    ):hover {
    box-shadow: var(--fjord-shadow-raised);
  }
  .fjord-button:not(:disabled):not([aria-disabled="true"]):active {
    box-shadow: var(--fjord-shadow-pressed);
  }
  .fjord-button[data-variant="primary"]:not(:disabled):not(
      [aria-disabled="true"]
    ):active {
    background: var(--fjord-button-primary-pressed-background);
    border-color: var(--fjord-button-primary-pressed-background);
  }
  .fjord-button[data-variant="secondary"]:not(:disabled):not(
      [aria-disabled="true"]
    ):active,
  .fjord-button[data-variant="ghost"]:not(:disabled):not(
      [aria-disabled="true"]
    ):active {
    background: var(--fjord-bg-active);
  }
  .fjord-button:focus-visible {
    outline: var(--fjord-button-primary-focus-width) solid
      var(--fjord-button-primary-focus-color);
    outline-offset: var(--fjord-focus-offset);
  }
  .fjord-button:disabled {
    background: var(--fjord-bg-surface);
    color: var(--fjord-button-primary-disabled-foreground);
    border-color: var(--fjord-line);
    cursor: not-allowed;
  }
  .fjord-button[aria-disabled="true"] {
    cursor: progress;
  }
  .spinner {
    display: inline-flex;
    animation: fjord-button-spin calc(var(--fjord-duration-slow) * 4) linear
      infinite;
  }
  @keyframes fjord-button-spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (any-pointer: coarse) {
    .fjord-button {
      min-block-size: var(--fjord-target-touch);
      min-inline-size: var(--fjord-target-touch);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation: none;
    }
  }
  @media (forced-colors: active) {
    .fjord-button {
      background: ButtonFace !important;
      color: ButtonText !important;
      border-color: ButtonText !important;
    }
    .fjord-button:disabled {
      color: GrayText !important;
      border-color: GrayText !important;
    }
    .fjord-button:focus-visible {
      outline-color: Highlight;
    }
  }
</style>
