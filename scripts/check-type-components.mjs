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
const directory = await mkdtemp(join(workspace, ".verification/type-ssr-"));
try {
  for (const name of ["Typography", "CodeBlock", "Kbd"]) {
    const source = await readFile(
      join(workspace, `src/lib/${name}.svelte`),
      "utf8",
    );
    await writeFile(
      join(directory, `${name}.js`),
      compile(source, { generate: "server", filename: `${name}.svelte` }).js
        .code,
    );
  }
  const fixture = `<script>
    import Typography from './Typography.js'; import CodeBlock from './CodeBlock.js'; import Kbd from './Kbd.js';
    let { code, segments, label } = $props();
  </script>
  <Typography as="h2" role="title-2">Section</Typography><Kbd>Ctrl</Kbd><CodeBlock {label} {code} {segments}/>`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const code = '// Æøå & <x>\nconst x = "=>";';
  const segments = [
    { text: "// Æøå & <x>\n", role: "comment" },
    { text: 'const x = "=>";', role: "text" },
  ];
  const props = { label: "Example", code, segments };
  const htmlFor = (value) => render(Fixture, { props: value }).body;
  const html = htmlFor(props).replace(/<!--.*?-->/gs, "");
  assert.match(html, /<h2 class="fjord-type-title-2">Section<\/h2>/);
  assert.match(html, /<kbd[^>]*>Ctrl<\/kbd>/);
  assert.match(html, /role="region" aria-label="Example" tabindex="0"/);
  assert.match(html, /data-syntax="comment">\/\/ Æøå &amp; &lt;x>/);
  assert.match(html, /role="status"/);
  assert.throws(() => htmlFor({ ...props, label: " " }), /requires a label/);
  assert.throws(
    () => htmlFor({ ...props, code: code + "!" }),
    /must match code exactly/,
  );
  console.log(
    "PASS semantic typography, keyboard markup, code text and segment integrity",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
