<script lang="ts">
  import {
    Alert,
    Button,
    Empty,
    Skeleton,
    Spinner,
    Table,
    ThemeScope,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/overlay-components.css";
  import "../lib/data-feedback.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let state = $state<"data" | "loading" | "empty" | "error">("data");
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Tables and feedback</h1>
      <p>Switch between examples. No data is sent.</p>
      <div class="settings">
        <label
          >Mode <select bind:value={mode}
            ><option value="night">Night</option><option value="day">Day</option
            ><option value="aurora">Aurora</option><option value="system"
              >System</option
            ></select
          ></label
        ><label
          >Identity <select bind:value={identity}
            ><option value="fjord">Fjord</option><option value="fern"
              >Fern</option
            ></select
          ></label
        >
      </div>
      <section class="fjord-panel fjord-stack" aria-label="Table example">
        <h2>Recent work</h2>
        <p>
          The table retains its headings and scrolls within its region on narrow
          screens.
        </p>
        <div class="actions">
          <Button
            label="Show data"
            variant="secondary"
            onclick={() => (state = "data")}
          /><Button
            label="Show loading"
            variant="secondary"
            onclick={() => (state = "loading")}
          /><Button
            label="Show empty"
            variant="secondary"
            onclick={() => (state = "empty")}
          /><Button
            label="Show error"
            variant="secondary"
            onclick={() => (state = "error")}
          />
        </div>
        {#if state === "data"}
          <Table id="recent-work" caption="Recent work">
            <thead
              ><tr
                ><th scope="col">Project</th><th scope="col">Status</th><th
                  scope="col">Details</th
                ></tr
              ></thead
            >
            <tbody
              ><tr
                ><th scope="row">Fjord</th><td>Ready</td><td
                  >A clear, readable entry.</td
                ></tr
              ><tr
                ><th scope="row">Notes</th><td>In progress</td><td
                  >A-long-unbroken-value-that-wraps-without-hiding-its-contents-abcdefghijklmnopqrstuvwxyz</td
                ></tr
              ></tbody
            >
          </Table>
        {:else if state === "loading"}
          <div aria-busy="true" class="loading">
            <Spinner label="Loading recent work" /><Skeleton
              width="medium"
            /><Skeleton /><Skeleton shape="block" />
          </div>
        {:else if state === "empty"}
          <Empty
            title="No work yet"
            description="There are no projects in this example."
            ><Button
              label="View sample projects"
              onclick={() => (state = "data")}
            /></Empty
          >
        {:else}
          <Alert
            title="Could not load work"
            message="Try again when you are ready."
            variant="danger"
            announcement="polite"
          />
          <div>
            <Button
              label="Try again"
              variant="secondary"
              onclick={() => (state = "data")}
            />
          </div>
        {/if}
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .settings,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--fjord-space-3);
  }
  .settings label {
    display: flex;
    align-items: center;
    gap: var(--fjord-space-2);
  }
  select {
    font: inherit;
    padding: var(--fjord-space-2);
    color: var(--fjord-ink);
    background: var(--fjord-bg-surface);
    border: 1px solid var(--fjord-line-strong);
  }
  .loading {
    display: grid;
    gap: var(--fjord-space-3);
  }
  h1,
  h2,
  p {
    margin: 0;
  }
</style>
