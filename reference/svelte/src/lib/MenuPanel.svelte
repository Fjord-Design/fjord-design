<script lang="ts">
  import { tick } from "svelte";
  import MenuPanel from "./MenuPanel.svelte";
  import type { MenuItem } from "./complete-types.js";
  let {
    label,
    items,
    onselect,
    onclose,
    onhorizontal,
  }: {
    label: string;
    items: MenuItem[];
    onselect: (id: string) => void;
    onclose: () => void;
    onhorizontal?: (direction: number) => void;
  } = $props();
  let panel: HTMLDivElement;
  let submenu = $state<string>();
  let search = "",
    last = 0;
  function buttons() {
    return Array.from(
      panel.querySelectorAll<HTMLButtonElement>(':scope > [role^="menuitem"]'),
    );
  }
  function first() {
    submenu = undefined;
    buttons()[0]?.focus();
  }
  export { first as focusFirst };
  async function child(item: MenuItem) {
    submenu = item.id;
    await tick();
    panel
      .querySelector<HTMLButtonElement>(`[data-submenu] [role^="menuitem"]`)
      ?.focus();
  }
  function activate(item: MenuItem) {
    if (item.disabled) return;
    if (item.children?.length) {
      void child(item);
      return;
    }
    onselect(item.id);
  }
  function key(e: KeyboardEvent) {
    if (
      e.target instanceof Element &&
      e.target.closest("[role=menu]") !== panel
    )
      return;
    const list = buttons(),
      i = list.indexOf(e.target as HTMLButtonElement);
    let n = i;
    if (e.key === "ArrowDown") n = (i + 1) % list.length;
    else if (e.key === "ArrowUp") n = (i - 1 + list.length) % list.length;
    else if (e.key === "Home") n = 0;
    else if (e.key === "End") n = list.length - 1;
    else if (e.key === "Escape" || e.key === "ArrowLeft") {
      e.preventDefault();
      if (e.key === "ArrowLeft" && onhorizontal) onhorizontal(-1);
      else onclose();
      return;
    } else if (e.key === "Tab") {
      onclose();
      return;
    } else if (e.key === "ArrowRight") {
      const item = items[i];
      if (item?.children?.length) {
        e.preventDefault();
        void child(item);
      } else if (onhorizontal) {
        e.preventDefault();
        onhorizontal(1);
      }
      return;
    } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
      const now = Date.now();
      search = now - last > 700 ? e.key : search + e.key;
      last = now;
      n = items.findIndex((item) =>
        item.label.toLocaleLowerCase().startsWith(search.toLocaleLowerCase()),
      );
      if (n < 0) return;
    } else return;
    e.preventDefault();
    list[n]?.focus();
  }
</script>

<div
  bind:this={panel}
  class="fjord-menu-panel"
  role="menu"
  tabindex="-1"
  aria-label={label}
  onkeydown={key}
>
  {#each items as item}<button
      type="button"
      role={item.kind === "checkbox"
        ? "menuitemcheckbox"
        : item.kind === "radio"
          ? "menuitemradio"
          : "menuitem"}
      aria-checked={item.kind && item.kind !== "action"
        ? !!item.checked
        : undefined}
      aria-disabled={item.disabled || undefined}
      aria-haspopup={item.children?.length ? "menu" : undefined}
      aria-expanded={item.children?.length ? submenu === item.id : undefined}
      tabindex="-1"
      onclick={() => activate(item)}
      ><span>{item.checked ? "✓ " : ""}{item.label}</span
      >{#if item.shortcut}<kbd>{item.shortcut}</kbd
        >{/if}{#if item.children?.length}<span aria-hidden="true">▸</span
        >{/if}</button
    >
    {#if item.children?.length && submenu === item.id}<div data-submenu>
        <MenuPanel
          label={item.label}
          items={item.children}
          {onselect}
          onclose={async () => {
            submenu = undefined;
            await tick();
            buttons()[items.indexOf(item)]?.focus();
          }}
        />
      </div>{/if}{/each}
</div>
