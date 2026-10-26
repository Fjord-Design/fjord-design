<script lang="ts">
  import { Icon, type IconName } from "../lib/index.js";
  import { iconPaths } from "../lib/icons.generated.js";
  import "../../../../dist/foundations.css";
  let { mode = "night" }: { mode?: "night" | "day" | "aurora" } = $props();
</script>

<section
  class="fjord-foundation fjord-type-body"
  data-fjord-mode={mode}
  aria-label="Icon foundation"
>
  <div class="fjord-page fjord-stack">
    <h1 class="fjord-type-title-1">Phosphor Fill</h1>
    <p>Shared solid icons. Text gives actions their accessible name.</p>
    <div class="icons">
      {#each Object.keys(iconPaths) as name}
        <span><Icon name={name as IconName} /><span>{name}</span></span>
      {/each}
    </div>
    <button class="fjord-control"
      ><Icon name="add" size="control" />Create theme</button
    >
    <p>
      Standalone meaning: <Icon
        name="info"
        decorative={false}
        label="Information"
      />
    </p>
  </div>
</section>

<style>
  .icons {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
  }
  .icons > span {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
