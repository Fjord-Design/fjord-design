<script lang="ts">
  import {
    Typography,
    CodeBlock,
    Kbd,
    ThemeScope,
    type CodeSegment,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/type-components.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  const parts: CodeSegment[] = [
    {
      role: "comment",
      text: "// Readable comments keep their original markers.\n",
    },
    { role: "keyword", text: "const" },
    { role: "text", text: " " },
    { role: "variable", text: "greeting" },
    { role: "operator", text: " = " },
    { role: "string", text: '"Hej, Fjord!"' },
    { role: "punctuation", text: ";\n" },
    {
      role: "comment",
      text: "// A long line keeps its characters and scrolls within the code region: abcdefghijklmnopqrstuvwxyz0123456789",
    },
  ];
  const code = parts.map((part) => part.text).join("");
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <Typography as="h1" role="title-1">Typography and code</Typography>
      <Typography as="p" role="lead"
        >Readable text, source code and keyboard shortcuts.</Typography
      >
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
            ><option value="work">Work</option><option value="gaming"
              >Gaming</option
            ></select
          ></label
        >
      </div>
      <section class="fjord-panel fjord-stack">
        <Typography as="h2" role="title-2">Code example</Typography><CodeBlock
          label="JavaScript example"
          {code}
          segments={parts}
        /><Typography
          >On macOS, save with <Kbd>⌘</Kbd> + <Kbd>S</Kbd>. On Windows and
          Linux, use <Kbd>Ctrl</Kbd> + <Kbd>S</Kbd>.</Typography
        >
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
</style>
