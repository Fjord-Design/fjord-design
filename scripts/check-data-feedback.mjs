import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const workspace = join(root, "reference/svelte");
const require = createRequire(join(workspace, "package.json"));
const { compile } = require("svelte/compiler");
const typescript = require("typescript");
const { render } = await import(
  pathToFileURL(require.resolve("svelte/server")).href
);
await mkdir(join(workspace, ".verification"), { recursive: true });
const directory = await mkdtemp(join(workspace, ".verification/data-ssr-"));
try {
  for (const name of ["Table", "Empty", "Skeleton", "Spinner", "Icon"]) {
    let source = await readFile(
      join(workspace, `src/lib/${name}.svelte`),
      "utf8",
    );
    source = source.replace(
      'import Icon from "./Icon.svelte";',
      'import Icon from "./Icon.js";',
    );
    await writeFile(
      join(directory, `${name}.js`),
      compile(source, { generate: "server", filename: `${name}.svelte` }).js
        .code,
    );
  }
  const icons = await readFile(
    join(workspace, "src/lib/icons.generated.ts"),
    "utf8",
  );
  await writeFile(
    join(directory, "icons.generated.js"),
    typescript.transpile(icons, {
      module: typescript.ModuleKind.ESNext,
      target: typescript.ScriptTarget.ES2022,
    }),
  );
  const fixture = `<script>
    import Table from './Table.js'; import Empty from './Empty.js'; import Skeleton from './Skeleton.js'; import Spinner from './Spinner.js';
    let { caption, title, loading } = $props();
  </script>
  <Table id="items" {caption}><thead><tr><th scope="col">Name</th></tr></thead><tbody><tr><th scope="row">Fjord</th></tr></tbody></Table>
  <Empty {title} description="Create an item." />
  <Skeleton shape="block" /><Spinner label={loading} />`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const htmlFor = (props) => render(Fixture, { props }).body;
  const valid = {
    caption: "Recent work",
    title: "No work",
    loading: "Loading",
  };
  const html = htmlFor(valid);
  assert.match(
    html,
    /role="region"[^>]*aria-labelledby="items-caption"[^>]*tabindex="0"/,
  );
  assert.match(html, /<caption id="items-caption">Recent work<\/caption>/);
  assert.match(html, /<th scope="col">Name<\/th>/);
  assert.match(html, /<th scope="row">Fjord<\/th>/);
  assert.match(html, /class="fjord-skeleton"[^>]*aria-hidden="true"/);
  assert.match(html, /class="fjord-spinner" role="status"/);
  assert.throws(() => htmlFor({ ...valid, caption: " " }), /ID and a caption/);
  assert.throws(
    () => htmlFor({ ...valid, title: " " }),
    /title and description/,
  );
  assert.throws(() => htmlFor({ ...valid, loading: " " }), /non-empty label/);
  console.log(
    "PASS table headings, named scroll region, empty and loading semantics",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
