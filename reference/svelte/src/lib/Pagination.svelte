<script lang="ts">
  let {
    label = "Pagination",
    total,
    page = $bindable(1),
    pageSize = $bindable(10),
    sizes = [10, 25, 50],
  }: {
    label?: string;
    total: number;
    page?: number;
    pageSize?: number;
    sizes?: number[];
  } = $props();
  const pages = $derived.by(() => {
    if (
      !Number.isInteger(total) ||
      total < 0 ||
      !Number.isInteger(pageSize) ||
      pageSize <= 0 ||
      sizes.some((s) => !Number.isInteger(s) || s <= 0)
    )
      throw new Error(
        "Pagination needs non-negative total and positive integer sizes.",
      );
    return Math.max(1, Math.ceil(total / pageSize));
  });
  const current = $derived(Math.max(1, Math.min(pages, Math.floor(page) || 1)));
  $effect(() => {
    if (page !== current) page = current;
  });
</script>

<nav class="fjord-widget fjord-pagination" aria-label={label}>
  <button
    type="button"
    aria-label="Previous page"
    disabled={current <= 1 || !total}
    onclick={() => (page = current - 1)}>Previous</button
  ><span role="status"
    >{total ? `Page ${current} of ${pages}` : "No results"}</span
  ><button
    type="button"
    aria-label="Next page"
    disabled={current >= pages || !total}
    onclick={() => (page = current + 1)}>Next</button
  ><label
    >Rows per page <select
      value={pageSize}
      onchange={(e) => {
        pageSize = Number(e.currentTarget.value);
        page = 1;
      }}
      >{#each [...new Set( [...sizes, pageSize] )].sort((a, b) => a - b) as size}<option
          value={size}>{size}</option
        >{/each}</select
    ></label
  >
</nav>
