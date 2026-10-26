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
const directory = await mkdtemp(join(workspace, ".verification/choices-ssr-"));
try {
  for (const name of ["Checkbox", "Switch", "RadioGroup"]) {
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
    import Checkbox from './Checkbox.js'; import Switch from './Switch.js'; import RadioGroup from './RadioGroup.js';
    let { label, options } = $props();
  </script>
  <Checkbox {label} name="agree" value="yes" required checked />
  <Switch label="Enable updates" name="updates" value="on" checked />
  <RadioGroup id="period" name="period" label="How often?" {options} required help="Choose one" error="Select a period" value="day" />`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const renderFixture = (props) => render(Fixture, { props }).body;
  const options = [
    { value: "day", label: "Daily" },
    { value: "week", label: "Weekly", disabled: true },
  ];
  const html = renderFixture({ label: "Accept", options });
  assert.match(html, /type="checkbox"[^>]*required[^>]*checked/);
  assert.match(html, /role="switch"/);
  assert.match(html, /role="radiogroup"[^>]*aria-invalid="true"/);
  assert.match(html, /aria-describedby="period-help period-error"/);
  assert.match(html, /<legend>How often\?/);
  assert.match(html, /name="period" value="day"/);
  assert.match(html, /name="period" value="week" disabled/);
  assert.throws(
    () => renderFixture({ label: " ", options }),
    /non-empty label/,
  );
  assert.throws(
    () => renderFixture({ label: "Accept", options: [options[0], options[0]] }),
    /unique non-empty/,
  );
  assert.throws(
    () => renderFixture({ label: "Accept", options: [] }),
    /unique non-empty/,
  );
  console.log(
    "PASS native choice semantics, group association, validation and form attributes",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
