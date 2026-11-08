<script lang="ts">
  import type { DataColumn, DataRow } from "./complete-types.js";
  let {
    label,
    columns,
    rows,
    selected = $bindable<string[]>([]),
    selectable = true,
    loading = false,
    error,
    onretry,
  }: {
    label: string;
    columns: DataColumn[];
    rows: DataRow[];
    selected?: string[];
    selectable?: boolean;
    loading?: boolean;
    error?: string;
    onretry?: () => void;
  } = $props();
  let query = $state(""),
    sortKey = $state(""),
    ascending = $state(true);
  const visible = $derived.by(() => {
    if (new Set(rows.map((r) => r.id)).size !== rows.length)
      throw new Error("DataTable requires unique row IDs.");
    const filtered = rows.filter((row) =>
      columns.some((c) =>
        String(row[c.key] ?? "")
          .toLocaleLowerCase()
          .includes(query.toLocaleLowerCase()),
      ),
    );
    return sortKey
      ? [...filtered].sort((a, b) => {
          const av = a[sortKey],
            bv = b[sortKey];
          return (
            (typeof av === "number" && typeof bv === "number"
              ? av - bv
              : String(av ?? "").localeCompare(String(bv ?? ""), undefined, {
                  numeric: true,
                })) * (ascending ? 1 : -1)
          );
        })
      : filtered;
  });
  function choose(id: string) {
    selected = selected.includes(id)
      ? selected.filter((v) => v !== id)
      : [...selected, id];
  }
  function all() {
    const ids = visible.map((row) => row.id);
    selected = ids.every((id) => selected.includes(id))
      ? selected.filter((id) => !ids.includes(id))
      : [...new Set([...selected, ...ids])];
  }
</script>

<div class="fjord-widget fjord-data-table">
  <label>Filter {label}<input type="search" bind:value={query} /></label>
  <div role="status">
    {loading
      ? "Loading…"
      : error
        ? "Could not load data"
        : `${visible.length} rows; ${selected.length} selected${sortKey ? `. Sorted by ${columns.find((column) => column.key === sortKey)?.label ?? sortKey}, ${ascending ? "ascending" : "descending"}.` : ""}`}
  </div>
  {#if error}<div role="alert" aria-live="assertive" aria-atomic="true">
      <p>{error}</p>
    </div>
    {#if onretry}<button type="button" onclick={onretry}>Try again</button
      >{/if}{/if}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (Named scroll regions need keyboard scrolling, as in the existing Table component.) -->
  <div
    class="fjord-table-scroll"
    tabindex="0"
    role="region"
    aria-label={`${label}, scrollable table`}
    aria-busy={loading}
  >
    <table>
      <caption>{label}</caption><thead
        ><tr
          >{#if selectable}<th scope="col"
              ><input
                type="checkbox"
                aria-label="Select all filtered rows"
                checked={visible.length > 0 &&
                  visible.every((row) => selected.includes(row.id))}
                indeterminate={visible.some((row) =>
                  selected.includes(row.id),
                ) && !visible.every((row) => selected.includes(row.id))}
                disabled={!visible.length || loading}
                onchange={all}
              /></th
            >{/if}{#each columns as column}<th
              scope="col"
              aria-sort={sortKey === column.key
                ? ascending
                  ? "ascending"
                  : "descending"
                : undefined}
              >{#if column.sortable}<button
                  type="button"
                  onclick={() => {
                    if (sortKey === column.key) ascending = !ascending;
                    else {
                      sortKey = column.key;
                      ascending = true;
                    }
                  }}
                  >{column.label}{sortKey === column.key
                    ? ascending
                      ? " ↑"
                      : " ↓"
                    : ""}</button
                >{:else}{column.label}{/if}</th
            >{/each}</tr
        ></thead
      ><tbody
        >{#each visible as row (row.id)}<tr
            aria-selected={selectable ? selected.includes(row.id) : undefined}
            >{#if selectable}<td
                ><input
                  type="checkbox"
                  aria-label={`Select row ${row.id}`}
                  checked={selected.includes(row.id)}
                  disabled={loading}
                  onchange={() => choose(row.id)}
                /></td
              >{/if}{#each columns as column}<td>{row[column.key] ?? "—"}</td
              >{/each}</tr
          >{/each}{#if !visible.length && !loading && !error}<tr
            ><td colspan={columns.length + (selectable ? 1 : 0)}>No results</td
            ></tr
          >{/if}</tbody
      >
    </table>
  </div>
</div>
