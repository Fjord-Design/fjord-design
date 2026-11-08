<script lang="ts">
  import { tick, untrack } from "svelte";
  import PanelDialog from "./PanelDialog.svelte";
  import type { MenuItem } from "./complete-types.js";
  let {
    id,
    label,
    items,
    open = $bindable(false),
    shortcut = "Control+k",
    onselect,
  }: {
    id: string;
    label: string;
    items: MenuItem[];
    open?: boolean;
    shortcut?: string;
    onselect: (id: string) => void;
  } = $props();
  let query = $state(""),
    active = $state(0);
  let input: HTMLInputElement;
  const filtered = $derived(
    items.filter((i) =>
      i.label.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
    ),
  );
  const enabled = $derived(
    filtered
      .map((i, index) => (!i.disabled ? index : -1))
      .filter((i) => i >= 0),
  );
  $effect(() => {
    if (open) {
      query = "";
      active = untrack(() => items.findIndex((item) => !item.disabled));
      void tick().then(() => input?.focus());
    }
  });
  function choose(item: MenuItem) {
    if (item.disabled) return;
    onselect(item.id);
    open = false;
  }
  function key(e: KeyboardEvent) {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const index = enabled.indexOf(active);
      active =
        enabled[
          (index + (e.key === "ArrowUp" ? -1 : 1) + enabled.length) %
            enabled.length
        ] ?? -1;
    } else if (e.key === "Enter" && filtered[active]) {
      e.preventDefault();
      choose(filtered[active]);
    }
  }
  function matches(e: KeyboardEvent, combination: string) {
    const parts = combination.toLowerCase().split("+");
    return (
      e.key.toLowerCase() === parts.at(-1) &&
      e.ctrlKey === parts.includes("control") &&
      e.metaKey === parts.includes("meta") &&
      e.altKey === parts.includes("alt") &&
      e.shiftKey === parts.includes("shift")
    );
  }
  function globalKey(e: KeyboardEvent) {
    if (matches(e, shortcut)) {
      e.preventDefault();
      open = !open;
    } else if (open) {
      const item = items.find((i) => i.shortcut && matches(e, i.shortcut));
      if (item && !item.disabled) {
        e.preventDefault();
        choose(item);
      }
    }
  }
</script>

<svelte:window onkeydown={globalKey} />
<PanelDialog {id} title={label} bind:open
  ><label for={`${id}-search`}>Search commands</label><input
    bind:this={input}
    id={`${id}-search`}
    role="combobox"
    aria-expanded={true}
    aria-controls={`${id}-results`}
    aria-activedescendant={filtered[active]
      ? `${id}-command-${active}`
      : undefined}
    aria-autocomplete="list"
    value={query}
    oninput={(e) => {
      query = e.currentTarget.value;
      active = enabled[0] ?? -1;
    }}
    onkeydown={key}
  />
  <div id={`${id}-results`} role="listbox" aria-label={label}>
    {#each filtered as item, index}<button
        type="button"
        tabindex="-1"
        role="option"
        id={`${id}-command-${index}`}
        aria-selected={active === index}
        aria-disabled={item.disabled || undefined}
        onpointerdown={(e) => e.preventDefault()}
        onclick={() => choose(item)}
        >{item.group ? `${item.group}: ` : ""}{item.label}
        {item.shortcut ?? ""}</button
      >{/each}
  </div>
  {#if !filtered.length}<p role="status">No commands found</p>{/if}</PanelDialog
>
