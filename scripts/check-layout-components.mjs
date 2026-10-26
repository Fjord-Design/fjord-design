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
const directory = await mkdtemp(join(workspace, ".verification/layout-ssr-"));
try {
  for (const name of ["Card", "Separator", "Tabs"]) {
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
    import Card from './Card.js'; import Separator from './Separator.js'; import Tabs from './Tabs.js';
    let { items } = $props();
  </script>
  <Card as="article"><Separator /><Separator decorative={false} orientation="vertical" />
    <Tabs id="group" label="Sections" {items} value="two" activation="manual">
      {#snippet children(panel)}<p>Content: {panel}</p>{/snippet}
    </Tabs>
  </Card>`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const htmlFor = (items) => render(Fixture, { props: { items } }).body;
  const items = [
    { value: "one", label: "First" },
    { value: "two", label: "Second" },
    { value: "three", label: "Unavailable", disabled: true },
  ];
  const html = htmlFor(items);
  assert.match(html, /<article class="fjord-card"/);
  assert.match(html, /<hr[^>]*aria-hidden="true"/);
  assert.match(html, /role="separator" aria-orientation="vertical"/);
  assert.match(html, /role="tablist" aria-label="Sections"/);
  assert.match(
    html,
    /id="group-tab-1"[^>]*aria-controls="group-panel-1"[^>]*aria-selected="true"/,
  );
  assert.match(html, /id="group-panel-1" aria-labelledby="group-tab-1"/);
  assert.match(html, /id="group-panel-0"[^>]*hidden/);
  assert.match(html, /id="group-tab-2"[^>]*disabled/);
  assert.throws(() => htmlFor([]), /unique non-empty/);
  assert.throws(() => htmlFor([items[0], items[0]]), /unique non-empty/);
  assert.throws(
    () => htmlFor([{ value: "x", label: "Disabled", disabled: true }]),
    /at least one enabled/,
  );
  console.log(
    "PASS cards, separator semantics, tab and panel links, selection and invalid input",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
