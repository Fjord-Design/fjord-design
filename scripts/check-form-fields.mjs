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
const directory = await mkdtemp(join(workspace, ".verification/fields-ssr-"));
try {
  for (const name of ["Field", "Label", "Input", "Textarea", "NativeSelect"]) {
    let source = await readFile(
      join(workspace, `src/lib/${name}.svelte`),
      "utf8",
    );
    source = source.replace('import "./form-controls.css";', "");
    source = source.replace(
      'import Label from "./Label.svelte";',
      'import Label from "./Label.js";',
    );
    const compiled = compile(source, {
      generate: "server",
      filename: `${name}.svelte`,
    });
    await writeFile(join(directory, `${name}.js`), compiled.js.code);
  }
  const fixture = `<script>
    import Field from './Field.js'; import Input from './Input.js';
    import Textarea from './Textarea.js'; import NativeSelect from './NativeSelect.js';
    let { id, label, error } = $props();
  </script>
  <Field {id} {label} required help="Help & detail" {error}>
    {#snippet children(control)}<Input {...control} name="email" type="email" autocomplete="email" value="person@example.com" />{/snippet}
  </Field>
  <Textarea id="note" aria-label="Note" readonly value="Read only" />
  <NativeSelect id="mode" aria-label="Mode" value="day"><option value="day">Day</option></NativeSelect>`;
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const renderField = (props) => render(Fixture, { props }).body;
  const html = renderField({
    id: "email",
    label: "Email",
    error: "Enter an email",
  });
  assert.match(html, /for="email"/);
  assert.match(html, /id="email"[^>]*required/);
  assert.match(html, /aria-invalid="true"/);
  assert.match(html, /aria-describedby="email-help email-error"/);
  assert.match(html, /id="email-help"[^>]*>Help &amp; detail/);
  assert.match(html, /id="email-error"[^>]*>Enter an email/);
  assert.match(html, /autocomplete="email"/);
  assert.match(html, /<textarea[^>]*readonly/);
  assert.match(html, /<select[^>]*id="mode"/);
  const valid = renderField({ id: "email", label: "Email", error: "" });
  assert.doesNotMatch(valid, /aria-invalid/);
  assert.match(valid, /aria-describedby="email-help"/);
  assert.throws(
    () => renderField({ id: "bad id", label: "Email" }),
    /without whitespace/,
  );
  assert.throws(
    () => renderField({ id: "email", label: " " }),
    /non-empty label/,
  );
  console.log(
    "PASS form fields render native controls, label associations, help, errors and required states",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
