import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const workspace = join(root, "reference/svelte");
const require = createRequire(join(workspace, "package.json"));
const { compile } = require("svelte/compiler");
const { render } = await import(
  pathToFileURL(require.resolve("svelte/server")).href
);
await mkdir(join(workspace, ".verification"), { recursive: true });
const directory = await mkdtemp(join(workspace, ".verification/overlays-ssr-"));
try {
  for (const name of ["Dialog", "Alert", "Tooltip"]) {
    const source = await readFile(
      join(workspace, `src/lib/${name}.svelte`),
      "utf8",
    );
    const compiled = compile(source, {
      generate: "server",
      filename: `${name}.svelte`,
    });
    await writeFile(join(directory, `${name}.js`), compiled.js.code);
  }
  const fixture = `<script>
    import Dialog from './Dialog.js'; import Alert from './Alert.js'; import Tooltip from './Tooltip.js';
    let { title, text } = $props();
  </script>
  <Dialog id="modal" {title} description="Details" closeLabel="Close"><p>Body</p></Dialog>
  <Alert title="Update" message="Saved" announcement="polite" />
  <Alert title="Problem" message="Retry" announcement="assertive" variant="danger" />
  <Tooltip id="hint" {text}>{#snippet children(attributes)}<button {...attributes}>Action</button>{/snippet}</Tooltip>`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const htmlFor = (props) => render(Fixture, { props }).body;
  const html = htmlFor({ title: "Review", text: "Extra information" });
  assert.match(
    html,
    /<dialog[^>]*aria-modal="true"[^>]*aria-labelledby="modal-title"[^>]*aria-describedby="modal-description"/,
  );
  assert.match(html, /id="modal-title"[^>]*>Review/);
  assert.match(html, /role="status"/);
  assert.match(html, /role="alert"/);
  assert.match(html, /aria-describedby="hint"/);
  assert.match(
    html,
    /id="hint"[^>]*role="tooltip"|role="tooltip"[^>]*id="hint"/,
  );
  assert.throws(
    () => htmlFor({ title: " ", text: "Extra" }),
    /ID, title and close label/,
  );
  assert.throws(() => htmlFor({ title: "Review", text: " " }), /ID and text/);
  console.log(
    "PASS dialog naming, live-region roles and tooltip description associations",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
