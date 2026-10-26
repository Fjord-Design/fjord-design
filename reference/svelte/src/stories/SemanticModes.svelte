<script lang="ts">
  import { ThemeScope, type ModePreference } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
</script>

<ThemeScope {mode} {identity}>
  <section
    class="fjord-foundation fjord-semantic fjord-type-body"
    aria-label="Semantic modes"
  >
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">One contract, two identities</h1>
      <label
        >Mode <select bind:value={mode}
          ><option value="night">Night</option><option value="day">Day</option
          ><option value="aurora">Aurora</option><option value="system"
            >System</option
          ></select
        ></label
      >
      <label
        >Identity <select bind:value={identity}
          ><option value="fjord">Fjord</option><option value="fern">Fern</option
          ></select
        ></label
      >
      <button class="fjord-control semantic-primary">Primary action</button>
      <p class="semantic-help">
        Descriptions keep their contrast when the identity changes.
      </p>
      <p class="semantic-status" data-status="success">Success: ready to use</p>
      <ThemeScope mode="day" identity="fjord"
        ><div
          class="fjord-foundation fjord-semantic fjord-type-body nested"
          aria-label="Independent Day scope"
        >
          <p>Independent Day scope</p>
          <button class="fjord-control semantic-primary">Nested action</button>
        </div></ThemeScope
      >
    </div>
  </section>
</ThemeScope>

<style>
  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  select {
    font: inherit;
    padding: 0.5rem;
    background: var(--fjord-bg-surface);
    color: var(--fjord-ink);
    border: 1px solid var(--fjord-line-strong);
    max-width: 100%;
  }
  .nested {
    padding: 1rem;
  }
</style>
