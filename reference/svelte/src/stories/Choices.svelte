<script lang="ts">
  import {
    Button,
    Checkbox,
    RadioGroup,
    Switch,
    ThemeScope,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/choices.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let mixed = $state(true);
  let checked = $state(false);
  let accepted = $state(false);
  let notifications = $state(false);
  let frequency = $state<string | undefined>();
  let result = $state("");
  const options = [
    { value: "daily", label: "Daily" },
    { value: "weekly", label: "Weekly" },
    { value: "monthly", label: "Monthly", disabled: true },
  ];
  function submit(
    event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement },
  ) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    result = `Saved locally: frequency ${data.get("frequency")}; agreement ${data.get("agreement")}; notifications ${data.get("notifications") ?? "off"}. Nothing was sent.`;
  }
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Choice controls</h1>
      <p>
        Use the labels and keyboard to try each control. The form sends no data.
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
      <section class="fjord-panel fjord-stack" aria-label="Checkbox states">
        <h2>Checkbox</h2>
        <Checkbox
          label="Some items selected"
          bind:checked
          bind:indeterminate={mixed}
        />
        <p id="mixed-state">
          Checked: {checked ? "yes" : "no"}. Mixed: {mixed ? "yes" : "no"}.
        </p>
        <Checkbox label="Unavailable option" disabled checked />
        <p>
          A mixed checkbox represents a partial selection. The application
          calculates and updates that state.
        </p>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Settings example">
        <h2>Settings example</h2>
        <form class="form" onsubmit={submit} oninput={() => (result = "")}>
          <RadioGroup
            id="frequency"
            name="frequency"
            label="How often?"
            {options}
            required
            help="Choose one frequency."
            bind:value={frequency}
          />
          <Switch
            name="notifications"
            value="on"
            label="Enable notifications"
            bind:checked={notifications}
          />
          <Checkbox
            name="agreement"
            value="yes"
            label="Confirm this example"
            required
            bind:checked={accepted}
          />
          <div><Button type="submit" label="Check choices" /></div>
          <p id="choice-result" role="status">{result}</p>
        </form>
        <Switch label="Unavailable setting" disabled />
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .form {
    display: grid;
    gap: var(--fjord-space-3);
  }
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
  .settings select {
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
