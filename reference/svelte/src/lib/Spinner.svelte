<script lang="ts">
  let { label }: { label: string } = $props();
  const checkedLabel = $derived.by(() => {
    if (!label?.trim()) throw new Error("Spinner requires a non-empty label.");
    return label;
  });
</script>

<div class="fjord-spinner" role="status">
  <span class="fjord-spinner-mark" aria-hidden="true"></span><span
    >{checkedLabel}</span
  >
</div>
