<script lang="ts">
  import type { Snippet } from "svelte";
  import type { TabItem } from "./types.js";
  let {
    id,
    label,
    items,
    value = $bindable(),
    activation = "automatic",
    orientation = "horizontal",
    panelTabbable = true,
    children,
  }: {
    id: string;
    label: string;
    items: TabItem[];
    value?: string;
    activation?: "automatic" | "manual";
    orientation?: "horizontal" | "vertical";
    panelTabbable?: boolean;
    children: Snippet<[string]>;
  } = $props();
  let list: HTMLDivElement | undefined = $state();
  let focused: string | undefined = $state();
  const enabled = $derived.by(() => {
    if (!id || /\s/.test(id) || !label?.trim())
      throw new Error(
        "Tabs requires an ID without whitespace and a non-empty label.",
      );
    if (
      !items.length ||
      items.some((item) => !item.value?.trim() || !item.label?.trim()) ||
      new Set(items.map((item) => item.value)).size !== items.length
    )
      throw new Error("Tabs requires unique non-empty values and labels.");
    const available = items.filter((item) => !item.disabled);
    if (!available.length)
      throw new Error("Tabs requires at least one enabled tab.");
    return available;
  });
  const active = $derived(
    enabled.find((item) => item.value === value)?.value ?? enabled[0].value,
  );
  const focusValue = $derived(
    enabled.some((item) => item.value === focused) ? focused : active,
  );
  $effect(() => {
    if (value !== active) value = active;
    if (
      !list?.contains(document.activeElement) ||
      !enabled.some((item) => item.value === focused)
    )
      focused = active;
  });
  function focusTab(item: TabItem) {
    focused = item.value;
    if (activation === "automatic") value = item.value;
  }
  function move(
    event: KeyboardEvent & { currentTarget: EventTarget & HTMLButtonElement },
  ) {
    const previous = orientation === "horizontal" ? "ArrowLeft" : "ArrowUp";
    const next = orientation === "horizontal" ? "ArrowRight" : "ArrowDown";
    const current = enabled.findIndex(
      (item) => item.value === event.currentTarget.dataset.value,
    );
    let target = current;
    if (event.key === previous)
      target = (current - 1 + enabled.length) % enabled.length;
    else if (event.key === next) target = (current + 1) % enabled.length;
    else if (event.key === "Home") target = 0;
    else if (event.key === "End") target = enabled.length - 1;
    else return;
    event.preventDefault();
    const selectedIndex = items.findIndex(
      (item) => item.value === enabled[target].value,
    );
    list
      ?.querySelectorAll<HTMLButtonElement>('[role="tab"]')
      [selectedIndex]?.focus();
  }
  function leave(event: FocusEvent) {
    if (!list?.contains(event.relatedTarget as Node | null)) focused = active;
  }
</script>

<div class="fjord-tabs" data-orientation={orientation} data-active={active}>
  <div
    class="fjord-tablist"
    role="tablist"
    aria-label={label}
    aria-orientation={orientation}
    bind:this={list}
    onfocusout={leave}
  >
    {#each items as item, index (item.value)}
      <button
        type="button"
        role="tab"
        id={`${id}-tab-${index}`}
        data-value={item.value}
        aria-controls={`${id}-panel-${index}`}
        aria-selected={item.value === active}
        tabindex={item.value === focusValue ? 0 : -1}
        disabled={item.disabled}
        onfocus={() => focusTab(item)}
        onclick={() => (value = item.value)}
        onkeydown={move}>{item.label}</button
      >
    {/each}
  </div>
  <div class="fjord-tabpanels">
    {#each items as item, index (item.value)}
      <div
        role="tabpanel"
        id={`${id}-panel-${index}`}
        aria-labelledby={`${id}-tab-${index}`}
        hidden={item.value !== active}
        tabindex={panelTabbable ? 0 : undefined}
      >
        {@render children(item.value)}
      </div>
    {/each}
  </div>
</div>
