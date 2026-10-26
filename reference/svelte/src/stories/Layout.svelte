<script lang="ts">
  import {
    Card,
    Separator,
    Tabs,
    ThemeScope,
    type ModePreference,
    type TabItem,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/layout-components.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let manualValue = $state("overview");
  let autoValue = $state("one");
  const manualItems: TabItem[] = [
    { value: "overview", label: "Overview" },
    { value: "details", label: "A much longer title for the details tab" },
    { value: "disabled", label: "Unavailable", disabled: true },
    { value: "settings", label: "Settings" },
    { value: "notes", label: "Notes and next steps" },
  ];
  const autoItems: TabItem[] = [
    { value: "one", label: "First" },
    { value: "two", label: "Second" },
    { value: "three", label: "Third" },
  ];
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Cards and tabs</h1>
      <p>
        Move between tabs with arrow keys. The first example uses Enter or Space
        to activate.
      </p>
      <div class="settings">
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
            ></select
          ></label
        >
      </div>
      <Card as="article">
        <div class="stack">
          <h2>Manual activation</h2>
          <p>
            Arrow keys move focus; activate a tab to show its panel. Long tab
            lists scroll horizontally.
          </p>
          <Separator />
          <Tabs
            id="manual-tabs"
            label="Project information"
            items={manualItems}
            activation="manual"
            bind:value={manualValue}
          >
            {#snippet children(panel)}
              {#if panel === "overview"}<h3>Overview</h3>
                <p>A quiet surface for related information.</p>
                <label>Draft note <input value="Keep this note" /></label>
              {:else if panel === "details"}<h3>Details</h3>
                <p>Panel content is kept while you visit another tab.</p>
              {:else if panel === "settings"}<h3>Settings</h3>
                <p>Settings for the example.</p>
              {:else if panel === "notes"}<h3>Notes</h3>
                <p>The example does not save data.</p>
              {:else}<h3>Unavailable</h3>{/if}
            {/snippet}
          </Tabs>
        </div>
      </Card>
      <Card>
        <div class="stack">
          <h2>Automatic activation</h2>
          <p>
            Arrow keys move focus and reveal the next preloaded panel. This
            group is vertical.
          </p>
          <Separator decorative={false} />
          <Tabs
            id="automatic-tabs"
            label="Reference examples"
            items={autoItems}
            orientation="vertical"
            bind:value={autoValue}
          >
            {#snippet children(panel)}<h3>
                {panel === "one"
                  ? "First"
                  : panel === "two"
                    ? "Second"
                    : "Third"}
              </h3>
              <p>Selected panel: {panel}.</p>{/snippet}
          </Tabs>
          <div class="row">
            <span>One</span><Separator orientation="vertical" /><span>Two</span>
          </div>
        </div>
      </Card>
    </div>
  </main>
</ThemeScope>

<style>
  .stack {
    display: grid;
    gap: var(--fjord-space-4);
  }
  .settings,
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: stretch;
    gap: var(--fjord-space-3);
  }
  .settings label {
    display: flex;
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
  h1,
  h2,
  h3,
  p {
    margin: 0;
  }
  :global([role="tabpanel"]) h3 {
    margin-block-end: var(--fjord-space-2);
  }
  :global([role="tabpanel"]) p {
    margin-block-end: var(--fjord-space-3);
  }
</style>
