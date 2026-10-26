<script lang="ts">
  import type { Snippet } from "svelte";
  import Label from "./Label.svelte";
  import type { FieldControl } from "./types.js";
  let {
    id,
    label,
    help,
    error,
    required = false,
    disabled = false,
    children,
  }: {
    id: string;
    label: string;
    help?: string;
    error?: string;
    required?: boolean;
    disabled?: boolean;
    children: Snippet<[FieldControl]>;
  } = $props();
  const checkedId = $derived.by(() => {
    if (!id || /\s/.test(id))
      throw new Error("Field requires a non-empty ID without whitespace.");
    if (!label?.trim()) throw new Error("Field requires a non-empty label.");
    return id;
  });
  const hasHelp = $derived(Boolean(help?.trim()));
  const hasError = $derived(Boolean(error?.trim()));
  const control = $derived({
    id: checkedId,
    required,
    disabled,
    "aria-invalid": hasError ? ("true" as const) : undefined,
    "aria-describedby":
      [hasHelp && `${checkedId}-help`, hasError && `${checkedId}-error`]
        .filter(Boolean)
        .join(" ") || undefined,
  });
</script>

<div class="fjord-field">
  <Label for={checkedId}
    >{label}{#if required}<span aria-hidden="true"> *</span>{/if}</Label
  >
  {@render children(control)}
  {#if hasHelp}<p id={`${checkedId}-help`} class="help">{help}</p>{/if}
  {#if hasError}<p id={`${checkedId}-error`} class="error">{error}</p>{/if}
</div>

<style>
  .fjord-field {
    display: grid;
    gap: var(--fjord-space-1);
    min-inline-size: 0;
  }
  p {
    margin: 0;
    font-family: var(--fjord-button-primary-font-family);
    font-size: var(--fjord-button-primary-font-size);
    line-height: 1.5;
    overflow-wrap: anywhere;
  }
  .help {
    color: var(--fjord-ink-muted);
  }
  .error {
    color: var(--fjord-danger);
  }
  @media (forced-colors: active) {
    .help,
    .error {
      color: CanvasText;
    }
  }
</style>
