<script lang="ts">
  import {
    Button,
    Checkbox,
    CodeBlock,
    Field,
    Input,
    Label,
    ThemeScope,
    type Mode,
    type ModePreference,
  } from "../lib/index.js";
  import { watchMode } from "../lib/theme-preference.js";
  import tokenData from "../../../../dist/catalogue-tokens.json";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/form-controls.css";
  import "../lib/choices.css";
  import "../lib/type-components.css";
  type Entry = {
    type: string;
    value: unknown;
    original: unknown;
    source: string;
    hex?: string;
  };
  type Document = { tokens: Record<string, Entry> };
  type Identity = "fjord" | "fern";
  const data = tokenData as unknown as Record<Identity, Record<Mode, Document>>;
  const logoNight = new URL(
    "../../../../assets/brand/fjord-logo.svg",
    import.meta.url,
  ).href;
  const logoDay = new URL(
    "../../../../assets/brand/fjord-logo-day.svg",
    import.meta.url,
  ).href;
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: Identity } = $props();
  let systemMode = $state<Mode>("night");
  let density = $state<"comfortable" | "compact">("comfortable");
  let token = $state("semantic.color.accent");
  let search = $state("");
  let updates = $state(true);
  let sampleName = $state("Fjord workspace");
  let status = $state("");
  let attempt = 0;
  const resolvedMode = $derived(mode === "system" ? systemMode : mode);
  const entries = $derived(data[identity][resolvedMode].tokens);
  const entry = $derived(entries[token]);
  const names = $derived(
    Object.keys(entries).filter((key) => key.includes(search.toLowerCase())),
  );
  const swatches = $derived(
    Object.entries(entries).filter(
      ([key, value]) => key.startsWith("semantic.color.") && value.hex,
    ),
  );
  const example = $derived(
    `var(--fjord-token-${token.replaceAll(".", "-")}${entry?.type === "typography" ? "-font-family" : ""})`,
  );
  const code =
    "$ PYTHONPATH=src python3 -m fjord_tokens export --mode night --format wezterm\nExported night wezterm: dist/night.wezterm.toml\n\n~/fjord  main  ✓";
  const typeRoles = [
    "display",
    "title-1",
    "title-2",
    "title-3",
    "title-4",
    "lead",
    "body",
    "body-strong",
    "label",
    "small",
    "caption",
    "overline",
    "code",
    "code-small",
    "code-italic",
  ];
  $effect(() =>
    watchMode(mode, (value) => {
      systemMode = value;
    }),
  );
  async function copyExample() {
    const current = ++attempt;
    status = "";
    try {
      await navigator.clipboard.writeText(example);
      if (current === attempt) status = "Example copied";
    } catch {
      if (current === attempt)
        status = "Copy failed. Select the example and copy it manually.";
    }
  }
</script>

<ThemeScope {mode} {identity}>
  <main
    class="fjord-foundation fjord-semantic fjord-type-body"
    data-fjord-density={density}
  >
    <div class="fjord-page fjord-stack">
      <header class="brand">
        <img src={resolvedMode === "day" ? logoDay : logoNight} alt="Fjord" />
        <p>Design system reference</p>
      </header>
      <h1 class="fjord-type-title-1">Explore Fjord</h1>
      <p>Inspect the shared tokens and try a settings page in each mode.</p>
      <section
        class="fjord-panel fjord-stack"
        aria-labelledby="catalogue-settings"
      >
        <h2 id="catalogue-settings" class="fjord-type-title-2">Appearance</h2>
        <div class="controls">
          <label
            >Mode <select bind:value={mode}
              ><option value="night">Night</option><option value="day"
                >Day</option
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
          ><label
            >Density <select bind:value={density}
              ><option value="comfortable">Comfortable</option><option
                value="compact">Compact</option
              ></select
            ></label
          >
        </div>
        <p>These choices affect this example only and are not stored.</p>
        <Field
          id="catalogue-name"
          label="Workspace name"
          help="Visible to your team."
          >{#snippet children(control)}<Input
              {...control}
              bind:value={sampleName}
            />{/snippet}</Field
        >
        <Checkbox label="Notify me about changes" bind:checked={updates} />
        <div>
          <Button
            label="Save settings"
            onclick={() =>
              (status = "Example settings are ready. No data was saved.")}
          />
        </div>
      </section>
      <section class="fjord-panel fjord-stack" aria-labelledby="token-heading">
        <h2 id="token-heading" class="fjord-type-title-2">Inspect tokens</h2>
        <div>
          <Label for="catalogue-token-filter">Filter token names</Label><input
            id="catalogue-token-filter"
            type="search"
            bind:value={search}
          />
        </div>
        <label
          >Token <select bind:value={token} size="6"
            >{#each names as name}<option value={name}>{name}</option
              >{/each}</select
          ></label
        >
        {#if names.length === 0}<p>
            No matching tokens. Change the filter.
          </p>{/if}
        {#if entry}
          <dl>
            <dt>Name</dt>
            <dd><code>{token}</code></dd>
            <dt>Source</dt>
            <dd>{entry.source}</dd>
            <dt>Alias or source value</dt>
            <dd><code>{JSON.stringify(entry.original)}</code></dd>
            <dt>Resolved value</dt>
            <dd>
              <code
                >{JSON.stringify(entry.value)}{entry.hex
                  ? ` (${entry.hex})`
                  : ""}</code
              >
            </dd>
          </dl>
          <div class="controls">
            <code>{example}</code><Button
              label="Copy example"
              variant="secondary"
              onclick={copyExample}
            />
          </div>
        {/if}
        <p role="status">{status}</p>
      </section>
      <section class="fjord-panel fjord-stack" aria-labelledby="color-heading">
        <h2 id="color-heading" class="fjord-type-title-2">Semantic colors</h2>
        <div class="swatches">
          {#each swatches as [name, value]}<button
              type="button"
              class="swatch"
              onclick={() => {
                token = name;
                search = "";
              }}
              ><span class="sample" style:background={value.hex}></span><span
                >{name.replace("semantic.color.", "")}<br /><code
                  >{value.hex}</code
                ></span
              ></button
            >{/each}
        </div>
      </section>
      <section class="fjord-panel fjord-stack" aria-labelledby="type-heading">
        <h2 id="type-heading" class="fjord-type-title-2">Typography scale</h2>
        {#each typeRoles as role}<div>
            <span class={`fjord-type-${role}`}
              >Fjord: Klar tekst, god plads.</span
            ><br /><code>{role}</code>
          </div>{/each}
      </section>
      <section class="fjord-panel fjord-stack">
        <h2 class="fjord-type-title-2">Terminal example</h2>
        <p>The example uses the same surface and code tokens as the library.</p>
        <CodeBlock label="Terminal output" {code} />
      </section>
    </div>
  </main>
</ThemeScope>

<style>
  .brand {
    display: flex;
    align-items: center;
    gap: var(--fjord-space-4);
    flex-wrap: wrap;
  }
  .brand img {
    display: block;
    width: 160px;
    max-width: 100%;
    height: auto;
    margin: 16px;
  }
  .controls {
    display: flex;
    gap: var(--fjord-space-3);
    flex-wrap: wrap;
    align-items: center;
  }
  label {
    display: grid;
    gap: var(--fjord-space-2);
    min-width: 0;
  }
  select,
  input {
    font: inherit;
    color: var(--fjord-ink);
    background: var(--fjord-bg-surface);
    border: 1px solid var(--fjord-line-strong);
    border-radius: var(--fjord-radius-control);
    padding: var(--fjord-space-2);
    max-width: 100%;
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(12rem, 100%), 1fr));
    gap: var(--fjord-space-3);
  }
  .swatch {
    display: flex;
    align-items: center;
    gap: var(--fjord-space-2);
    text-align: left;
    font: inherit;
    color: var(--fjord-ink);
    background: var(--fjord-bg-surface);
    border: 1px solid var(--fjord-line-strong);
    border-radius: var(--fjord-radius-control);
    padding: var(--fjord-space-2);
    overflow-wrap: anywhere;
    cursor: pointer;
  }
  .sample {
    flex: none;
    width: 2rem;
    height: 2rem;
    border: 1px solid var(--fjord-line-strong);
  }
  dd {
    margin-inline: 0;
    overflow-wrap: anywhere;
  }
  dt {
    font-weight: 700;
  }
  h1,
  h2,
  p {
    margin: 0;
  }
</style>
