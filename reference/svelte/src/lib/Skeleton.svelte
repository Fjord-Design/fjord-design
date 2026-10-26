<script lang="ts">
  let {
    shape = "line",
    width = "full",
  }: { shape?: "line" | "block"; width?: "short" | "medium" | "full" } =
    $props();
</script>

<span
  class="fjord-skeleton"
  data-shape={shape}
  data-width={width}
  aria-hidden="true"
></span>
