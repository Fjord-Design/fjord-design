<script lang="ts">
  let {
    orientation = "horizontal",
    decorative = true,
  }: { orientation?: "horizontal" | "vertical"; decorative?: boolean } =
    $props();
</script>

{#if orientation === "horizontal"}
  <hr
    class="fjord-separator-component"
    aria-hidden={decorative ? "true" : undefined}
  />
{:else if decorative}
  <div
    class="fjord-separator-component"
    data-orientation="vertical"
    aria-hidden="true"
  ></div>
{:else}
  <div
    class="fjord-separator-component"
    data-orientation="vertical"
    role="separator"
    aria-orientation="vertical"
  ></div>
{/if}
