<script lang="ts">
  import {
    Badge,
    Button,
    ThemeScope,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let loading = $state(false);
  let submissions = $state(0);
  let activations = $state(0);
  let resets = $state(0);
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Button and Badge</h1>
      <p>Choose a theme and try the controls. The examples do not save data.</p>
      <div class="row">
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
            ><option value="fjord">Fjord</option><option value="fern"
              >Fern</option
            ><option value="work">Work</option><option value="gaming"
              >Gaming</option
            ></select
          ></label
        >
      </div>
      <section class="fjord-panel fjord-stack" aria-label="Button variants">
        <h2>Actions</h2>
        <div class="row">
          <Button label="Continue" icon="arrow-right" /><Button
            label="Edit"
            variant="secondary"
          /><Button label="Skip" variant="ghost" /><Button
            label="Delete"
            variant="destructive"
          /><Button label="Search" icon="search" iconOnly /><Button
            label="Unavailable"
            disabled
          />
        </div>
        <p>
          Use a named icon-only button when the icon is clear in context.
          Loading keeps focus and blocks repeated activation.
        </p>
        <div class="row">
          <Button
            label="Save changes"
            {loading}
            onclick={() => {
              activations++;
              loading = true;
            }}
          /><Button
            label="Finish example"
            variant="secondary"
            onclick={() => (loading = false)}
          />
        </div>
        <p id="activation-count">Activations: {activations}</p>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Native form example">
        <h2>Native form behaviour</h2>
        <form
          class="fjord-stack"
          onsubmit={(event) => {
            event.preventDefault();
            submissions++;
          }}
          onreset={() => resets++}
        >
          <label>Example input <input name="example" value="Fjord" /></label>
          <div class="row">
            <Button label="Submit" type="submit" {loading} /><Button
              label="Reset"
              type="reset"
              variant="secondary"
            /><Button label="Button" variant="ghost" />
          </div>
        </form>
        <p id="form-count">Submissions: {submissions}. Resets: {resets}.</p>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Badge variants">
        <h2>Status labels</h2>
        <div class="row">
          <Badge label="Draft" /><Badge
            label="Information"
            variant="info"
            icon="info"
          /><Badge label="Ready" variant="success" icon="success" /><Badge
            label="Attention"
            variant="warning"
            icon="warning"
          /><Badge label="Failed" variant="danger" icon="danger" />
        </div>
        <div class="long">
          <Badge
            variant="info"
            label="A long status label that wraps within its available space without truncating its meaning"
          />
        </div>
        <p>
          Badges are static labels. Use a button when the user can act on one.
        </p>
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--fjord-space-3);
  }
  label {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--fjord-space-2);
  }
  select,
  input {
    font: inherit;
    padding: var(--fjord-space-2);
    color: var(--fjord-ink);
    background: var(--fjord-bg-surface);
    border: 1px solid var(--fjord-line-strong);
    max-inline-size: 100%;
  }
  .long {
    max-inline-size: 14rem;
  }
  h1,
  h2,
  p {
    margin: 0;
  }
</style>
