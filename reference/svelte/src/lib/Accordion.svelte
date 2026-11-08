<script lang="ts">
  import type { Snippet } from "svelte";
  import type { DisclosureItem } from "./complete-types.js";
  let {
    id,
    items,
    value = $bindable<string[]>([]),
    multiple = false,
    children,
  }: {
    id: string;
    items: DisclosureItem[];
    value?: string[];
    multiple?: boolean;
    children: Snippet<[string]>;
  } = $props();
  let root: HTMLDivElement;
  const checked = $derived.by(() => {
    if (
      !id ||
      /\s/.test(id) ||
      new Set(items.map((i) => i.value)).size !== items.length ||
      items.some((i) => !i.label.trim() || !i.value)
    )
      throw new Error("Accordion needs an ID and unique named items.");
    return items;
  });
  function toggle(key: string) {
    value = value.includes(key)
      ? value.filter((v) => v !== key)
      : multiple
        ? [...value, key]
        : [key];
  }
  function move(e: KeyboardEvent) {
    const buttons = Array.from(
      root.querySelectorAll<HTMLButtonElement>(
        ":scope > section > h3 > button:not(:disabled)",
      ),
    );
    const i = buttons.indexOf(e.target as HTMLButtonElement);
    let n = i;
    if (e.key === "ArrowDown") n = (i + 1) % buttons.length;
    else if (e.key === "ArrowUp") n = (i - 1 + buttons.length) % buttons.length;
    else if (e.key === "Home") n = 0;
    else if (e.key === "End") n = buttons.length - 1;
    else return;
    e.preventDefault();
    buttons[n]?.focus();
  }
</script>

<div class="fjord-widget fjord-accordion" bind:this={root}>
  {#each checked as item, index (item.value)}<section>
      <h3>
        <button
          type="button"
          id={`${id}-heading-${index}`}
          aria-expanded={value.includes(item.value)}
          aria-controls={`${id}-panel-${index}`}
          disabled={item.disabled}
          onclick={() => toggle(item.value)}
          onkeydown={move}
          >{item.label}<span aria-hidden="true"
            >{value.includes(item.value) ? "−" : "+"}</span
          ></button
        >
      </h3>
      <div
        id={`${id}-panel-${index}`}
        role="region"
        aria-labelledby={`${id}-heading-${index}`}
        hidden={!value.includes(item.value)}
      >
        {@render children(item.value)}
      </div>
    </section>{/each}
</div>
