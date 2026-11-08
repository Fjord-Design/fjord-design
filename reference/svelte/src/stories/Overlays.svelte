<script lang="ts">
  import {
    Alert,
    Button,
    Dialog,
    ThemeScope,
    Tooltip,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/overlay-components.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let open = $state(false);
  let message = $state("No changes have been made.");
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Dialog, alerts and tooltips</h1>
      <p>These examples do not save or send data.</p>
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
            ><option value="work">Work</option><option value="gaming"
              >Gaming</option
            ></select
          ></label
        >
      </div>
      <section class="fjord-panel fjord-stack" aria-label="Dialog example">
        <h2>Modal dialog</h2>
        <p>
          Open it using the button. Escape or Close returns focus to the opener.
        </p>
        <div><Button label="Open dialog" onclick={() => (open = true)} /></div>
        <Dialog
          id="example-dialog"
          title="Review changes"
          description="This example does not make changes."
          closeLabel="Close"
          bind:open
        >
          <p>Dialog content and actions remain inside the modal.</p>
          <div>
            <Button
              label="Continue"
              variant="secondary"
              onclick={() => (open = false)}
            />
          </div>
        </Dialog>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Alert examples">
        <h2>Status messages</h2>
        <Alert title="Information" {message} announcement="polite" />
        <Alert
          title="Warning"
          message="Review the settings before continuing."
          variant="warning"
        />
        <Alert
          title="Error"
          message="This is an example message."
          variant="danger"
        />
        <div>
          <Button
            label="Update status"
            variant="secondary"
            onclick={() => (message = "The example status was updated.")}
          />
        </div>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Tooltip example">
        <h2>Supplementary help</h2>
        <p>
          Focus or hover over the control. On touch, tap to show or hide the
          help.
        </p>
        <div>
          <Tooltip id="example-tip" text="A copy is kept in this example only."
            >{#snippet children(attributes)}<Button
                {...attributes}
                label="Copy summary"
                variant="secondary"
              />{/snippet}</Tooltip
          >
        </div>
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .settings {
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
  h1,
  h2,
  p {
    margin: 0;
  }
</style>
