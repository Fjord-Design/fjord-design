<script lang="ts">
  import type { SelectOption } from "./complete-types.js";
  let {
    id,
    label,
    options,
    value = $bindable(""),
    name,
    disabled = false,
    required = false,
    loading = false,
    error,
    placeholder = "Choose…",
    editable = true,
  }: {
    id: string;
    label: string;
    options: SelectOption[];
    value?: string;
    name?: string;
    disabled?: boolean;
    required?: boolean;
    loading?: boolean;
    error?: string;
    placeholder?: string;
    editable?: boolean;
  } = $props();
  let invalid = $state(false);
  let open = $state(false),
    query = $state(""),
    active = $state(-1);
  let root: HTMLDivElement;
  let input = $state<HTMLInputElement>();
  let trigger = $state<HTMLButtonElement>();
  const selected = $derived(
    options.find((o) => o.value === value && !o.disabled),
  );
  const filtered = $derived(
    editable && open
      ? options.filter((o) =>
          o.label.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
        )
      : options,
  );
  $effect(() => {
    if (!open) query = selected?.label ?? "";
    if (input)
      input.setCustomValidity(
        required && !selected ? "Choose an available option." : "",
      );
  });
  function choose(option: SelectOption) {
    if (disabled || loading || error || option.disabled) return;
    invalid = false;
    value = option.value;
    query = option.label;
    open = false;
    (editable ? input : trigger)?.focus();
  }
  function move(e: KeyboardEvent) {
    const candidates = filtered
      .map((o, i) => (!o.disabled ? i : -1))
      .filter((i) => i >= 0);
    if (e.key === "Escape") {
      if (open) {
        e.preventDefault();
        open = false;
      }
      return;
    }
    if (e.key === "Tab") {
      open = false;
      return;
    }
    if (e.key === "Enter" && open) {
      e.preventDefault();
      if (filtered[active]) choose(filtered[active]);
      return;
    }
    if (
      !["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key) ||
      (!open && ["Home", "End"].includes(e.key))
    )
      return;
    e.preventDefault();
    if (!open) {
      open = true;
      active =
        e.key === "ArrowUp" ? (candidates.at(-1) ?? -1) : (candidates[0] ?? -1);
      return;
    }
    const i = candidates.indexOf(active);
    active =
      e.key === "Home"
        ? (candidates[0] ?? -1)
        : e.key === "End"
          ? (candidates.at(-1) ?? -1)
          : (candidates[
              (i + (e.key === "ArrowUp" ? -1 : 1) + candidates.length) %
                candidates.length
            ] ?? -1);
  }
  function outside(e: PointerEvent) {
    if (open && e.target instanceof Node && !root.contains(e.target))
      open = false;
  }
  function leave(e: FocusEvent) {
    if (e.relatedTarget instanceof Node && root.contains(e.relatedTarget))
      return;
    open = false;
  }
  let typeahead = "",
    last = 0;
  function selectKey(e: KeyboardEvent) {
    move(e);
    if (editable || e.key.length !== 1 || e.ctrlKey || e.metaKey) return;
    const now = Date.now();
    typeahead = now - last > 700 ? e.key : typeahead + e.key;
    last = now;
    const i = options.findIndex(
      (o) =>
        !o.disabled &&
        o.label.toLocaleLowerCase().startsWith(typeahead.toLocaleLowerCase()),
    );
    if (i >= 0) {
      e.preventDefault();
      open = true;
      active = i;
    }
  }
</script>

<svelte:window onpointerdown={outside} />
<div class="fjord-widget fjord-combobox" bind:this={root} onfocusout={leave}>
  <label for={id}>{label}{required ? " *" : ""}</label>
  {#if editable}<input
      bind:this={input}
      {id}
      type="text"
      role="combobox"
      aria-expanded={open}
      aria-controls={`${id}-list`}
      aria-autocomplete="list"
      aria-activedescendant={open && !loading && !error && !!filtered[active]
        ? `${id}-option-${active}`
        : undefined}
      aria-invalid={error || invalid ? "true" : undefined}
      aria-describedby={error ? `${id}-error` : undefined}
      {disabled}
      {required}
      {placeholder}
      value={query}
      oninput={(e) => {
        query = e.currentTarget.value;
        value = "";
        open = true;
        active = -1;
      }}
      onkeydown={move}
      onclick={() => {
        open = !open;
        active = -1;
      }}
    />
  {:else}<button
      bind:this={trigger}
      type="button"
      {id}
      role="combobox"
      aria-expanded={open}
      aria-controls={`${id}-list`}
      aria-activedescendant={open && !loading && !error && !!filtered[active]
        ? `${id}-option-${active}`
        : undefined}
      aria-required={required || undefined}
      aria-invalid={error || invalid ? "true" : undefined}
      aria-describedby={error ? `${id}-error` : undefined}
      {disabled}
      onclick={() => {
        open = !open;
        active = options.findIndex((o) => o.value === value);
      }}
      onkeydown={selectKey}
      >{selected?.label ?? placeholder}
      <span aria-hidden="true">▾</span></button
    >{/if}
  {#if editable}{#if name}<input type="hidden" {name} {value} {disabled} />{/if}
  {:else}<select
      class="fjord-select-proxy"
      aria-hidden="true"
      tabindex="-1"
      {name}
      {disabled}
      {required}
      {value}
      oninvalid={(event) => {
        event.preventDefault();
        invalid = true;
        trigger?.focus();
      }}
      onchange={(event) => {
        const option = options.find(
          (option) => option.value === event.currentTarget.value,
        );
        if (option) choose(option);
      }}
    >
      <option value="">Choose an option</option>
      {#each options as option}<option
          value={option.value}
          disabled={option.disabled}>{option.label}</option
        >{/each}
    </select>{#if invalid}<div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
      >
        <p>Choose an available option for {label}.</p>
      </div>{/if}{/if}
  <div
    id={`${id}-list`}
    class="fjord-listbox"
    role="listbox"
    aria-label={label}
    aria-busy={loading}
    hidden={!open}
  >
    {#if loading}<p role="status">Loading…</p>{:else if error}<p role="status">
        {error}
      </p>{:else if !filtered.length}<p role="status">
        No results
      </p>{:else}{#each filtered as option, i}<button
          type="button"
          tabindex="-1"
          id={`${id}-option-${i}`}
          role="option"
          aria-selected={option.value === value}
          aria-disabled={option.disabled || undefined}
          class:active={i === active}
          onpointerdown={(e) => e.preventDefault()}
          onclick={() => choose(option)}
          >{#if option.group}<small
              >{option.group}:
            </small>{/if}{option.label}{option.value === value
            ? " ✓"
            : ""}</button
        >{/each}{/if}
  </div>
  {#if error}<p id={`${id}-error`}>{error}</p>{/if}
</div>
