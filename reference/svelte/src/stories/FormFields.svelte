<script lang="ts">
  import { tick } from "svelte";
  import {
    Button,
    Field,
    Input,
    Textarea,
    NativeSelect,
    ThemeScope,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/form-controls.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let email = $state("");
  let message = $state("");
  let theme = $state("night");
  let submitted = $state(false);
  let result = $state("");
  let includedReadonly = $state(false);
  let includedDisabled = $state(false);
  let attempted = $state(false);
  let invalidEmail = $state(false);
  async function submit(
    event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement },
  ) {
    event.preventDefault();
    attempted = true;
    const emailControl = event.currentTarget.elements.namedItem(
      "email",
    ) as HTMLInputElement;
    invalidEmail = !emailControl.validity.valid;
    if (!event.currentTarget.checkValidity()) {
      submitted = false;
      await tick();
      emailControl.focus();
      return;
    }
    const data = new FormData(event.currentTarget);
    result = `Ready: ${data.get("email")}, ${data.get("theme")}. Nothing was sent.`;
    includedReadonly = data.has("readonly-example");
    includedDisabled = data.has("disabled-example");
    submitted = true;
  }
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1 class="fjord-type-title-1">Form fields</h1>
      <p>
        Labels, descriptions and errors stay connected to their controls. This
        example does not send data.
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
      <section class="fjord-panel fjord-stack" aria-label="Form example">
        <h2>Send a note</h2>
        <form
          id="example-form"
          class="form"
          novalidate
          onsubmit={submit}
          oninput={() => (submitted = false)}
          onchange={() => (submitted = false)}
        >
          <Field
            id="contact-email"
            label="Email"
            required
            help="Used only in this example."
            error={invalidEmail ? "Enter a valid email address." : undefined}
          >
            {#snippet children(control)}<Input
                {...control}
                name="email"
                type="email"
                autocomplete="email"
                bind:value={email}
                placeholder="you@example.com"
                oninput={(event) => {
                  if (attempted)
                    invalidEmail = !event.currentTarget.validity.valid;
                }}
              />{/snippet}
          </Field>
          <Field
            id="contact-message"
            label="Message"
            help="You can resize this field vertically."
          >
            {#snippet children(control)}<Textarea
                {...control}
                name="message"
                rows={4}
                bind:value={message}
              />{/snippet}
          </Field>
          <Field
            id="contact-theme"
            label="Preferred mode"
            help="Uses the system's native select menu."
          >
            {#snippet children(control)}<NativeSelect
                {...control}
                name="theme"
                bind:value={theme}
                ><option value="night">Night</option><option value="day"
                  >Day</option
                ><option value="aurora">Aurora</option></NativeSelect
              >{/snippet}
          </Field>
          <div><Button type="submit" label="Check form" /></div>
          <p
            role="status"
            id="form-result"
            data-readonly-included={includedReadonly}
            data-disabled-included={includedDisabled}
          >
            {submitted ? result : ""}
          </p>
        </form>
      </section>
      <section class="fjord-panel fjord-stack" aria-label="Other states">
        <h2>Other states</h2>
        <Field
          id="readonly-value"
          label="Read only"
          help="This value remains selectable and can be submitted."
        >
          {#snippet children(control)}<Input
              {...control}
              name="readonly-example"
              form="example-form"
              readonly
              value="A longer value that remains selectable without being editable"
            />{/snippet}
        </Field>
        <Field
          id="disabled-value"
          label="Unavailable"
          disabled
          help="The label and help remain readable."
        >
          {#snippet children(control)}<Input
              {...control}
              name="disabled-example"
              form="example-form"
              value="Not available"
            />{/snippet}
        </Field>
        <Field
          id="readonly-note"
          label="Read-only message"
          help="Its contents can be selected and copied."
        >
          {#snippet children(control)}<Textarea
              {...control}
              readonly
              value="A read-only multiline message."
              rows={2}
            />{/snippet}
        </Field>
        <Field
          id="disabled-choice"
          label="Unavailable choice"
          disabled
          help="A disabled select cannot be changed."
        >
          {#snippet children(control)}<NativeSelect {...control} value="night"
              ><option value="night">Night</option></NativeSelect
            >{/snippet}
        </Field>
        <p>
          Native select has no read-only state. Use static text when selection
          must not be editable.
        </p>
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .form {
    display: grid;
    gap: var(--fjord-space-4);
    max-inline-size: 36rem;
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
