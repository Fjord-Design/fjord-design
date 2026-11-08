import assert from "node:assert/strict";
import { createRequire, stripTypeScriptTypes } from "node:module";
import {
  readdir,
  readFile,
  writeFile,
  mkdtemp,
  rm,
  mkdir,
} from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
const root = fileURLToPath(new URL("../", import.meta.url)),
  workspace = join(root, "reference/svelte");
const require = createRequire(join(workspace, "package.json"));
const { compile } = require("svelte/compiler");
const { render } = await import(
  pathToFileURL(require.resolve("svelte/server")).href
);
await mkdir(join(workspace, ".verification"), { recursive: true });
const directory = await mkdtemp(join(workspace, ".verification/complete-"));
try {
  for (const filename of await readdir(join(workspace, "src/lib"))) {
    const source = await readFile(join(workspace, "src/lib", filename), "utf8");
    if (filename.endsWith(".svelte"))
      await writeFile(
        join(directory, filename.replace(".svelte", ".js")),
        compile(source, { generate: "server", filename })
          .js.code.replaceAll('.svelte"', '.js"')
          .replaceAll(".svelte'", ".js'"),
      );
    else if (filename.endsWith(".ts"))
      await writeFile(
        join(directory, filename.replace(".ts", ".js")),
        stripTypeScriptTypes(source)
          .replaceAll('.svelte"', '.js"')
          .replaceAll(".svelte'", ".js'"),
      );
    else if (filename.endsWith(".js"))
      await writeFile(join(directory, filename), source);
  }
  const fixture = (
    await readFile(join(workspace, "src/stories/Complete.svelte"), "utf8")
  )
    .replaceAll('from "../lib/index.js"', 'from "./index.js"')
    .replace(/import\s+["'][^"']+\.css["'];?/g, "");
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" })
      .js.code.replaceAll('.svelte"', '.js"')
      .replaceAll(".svelte'", ".js'"),
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const families = [
    "disclosures",
    "navigation",
    "overlays",
    "menus",
    "forms",
    "dates",
    "data",
    "feedback",
    "messages",
    "workflows",
    "layout",
  ];
  for (const family of families)
    for (const mode of ["night", "day", "aurora"])
      for (const identity of ["fjord", "fern", "work", "gaming"]) {
        const html = render(Fixture, {
          props: { family, mode, identity },
        }).body;
        assert.match(html, /Complete library/);
        assert.ok(!html.includes("[object Object]"), family);
        if (family === "disclosures") {
          assert.match(html, /aria-controls="complete-accordion-panel-0"/);
          assert.match(html, /id="complete-accordion-panel-0"[^>]*hidden/);
        }
        if (family === "menus") {
          assert.match(html, /role="combobox"/);
          assert.match(html, /aria-autocomplete="list"/);
          assert.match(html, /role="menuitemcheckbox"/);
        }
        if (family === "forms") {
          assert.match(html, /autocomplete="one-time-code"/);
          assert.match(html, /aria-pressed="false"/);
        }
        if (family === "dates") {
          assert.match(html, /2028-02-29/);
          assert.match(html, /type="date"/);
          assert.match(html, /role="grid"/);
        }
        if (family === "data") {
          assert.match(html, /Select all filtered rows/);
          assert.match(html, /Signed values, complete values/);
        }
        if (family === "messages") {
          assert.match(html, /datetime="2026-11-08T12:00:00\+01:00"/);
          assert.match(html, /Uploading long-project-document.pdf/);
        }
        if (family === "layout") {
          assert.match(html, /aria-roledescription="carousel"/);
          assert.match(html, /aria-valuenow="50"/);
          assert.match(html, /dir="rtl"/);
        }
      }
  const composedSource = (
    await readFile(join(workspace, "src/stories/Compositions.svelte"), "utf8")
  )
    .replaceAll('from "../lib/index.js"', 'from "./index.js"')
    .replace(/import\s+["'][^"']+\.css["'];?/g, "");
  await writeFile(
    join(directory, "Compositions.js"),
    compile(composedSource, {
      generate: "server",
      filename: "Compositions.svelte",
    }).js.code.replaceAll('.svelte"', '.js"'),
  );
  const { default: Compositions } = await import(
    pathToFileURL(join(directory, "Compositions.js")).href
  );
  for (const mode of ["night", "day", "aurora"])
    for (const identity of ["fjord", "fern", "work", "gaming"]) {
      const html = render(Compositions, { props: { mode, identity } }).body;
      for (const text of [
        "Project settings",
        "Workspace projects",
        "Workspace conversation",
        "Save settings",
        "Send reply",
      ])
        assert.ok(html.includes(text), text);
    }
  const { default: Calendar } = await import(
    pathToFileURL(join(directory, "Calendar.js")).href
  );
  const calendar = render(Calendar, {
    props: {
      label: "Disabled initial date",
      value: "2028-02-29",
      disabledDate: (date) => date === "2028-02-29",
    },
  }).body;
  const dateButtons = [
    ...calendar.matchAll(/<button[^>]*data-date="[^"]+"[^>]*>/g),
  ].map((match) => match[0]);
  const entryButtons = dateButtons.filter((button) =>
    button.includes('tabindex="0"'),
  );
  assert.equal(entryButtons.length, 1);
  assert.ok(!entryButtons[0].includes("disabled"));
  assert.ok(
    dateButtons
      .find((button) => button.includes('data-date="2028-02-29"'))
      .includes('tabindex="-1"'),
  );
  const { default: Slider } = await import(
    pathToFileURL(join(directory, "Slider.js")).href
  );
  assert.throws(
    () =>
      render(Slider, { props: { id: "invalid", label: "Invalid", value: NaN } })
        .body,
    /finite/,
  );
  const sliderCases = [
    { value: 23, min: 0, max: 100, step: 5, expected: 25 },
    { value: 22.5, min: 0, max: 100, step: 5, expected: 25 },
    { value: 20, min: 10, max: 20, step: 3, expected: 19 },
    { value: -20, min: 10, max: 20, step: 3, expected: 10 },
    { value: 0.3, min: 0.05, max: 0.5, step: 0.1, expected: 0.35 },
    { value: 0.35, min: 0.1, max: 0.6, step: 0.1, expected: 0.4 },
    { value: 0.35, min: 0.05, max: 0.35, step: 0.1, expected: 0.35 },
    { value: 0.3, min: 0, max: 0.35, step: 0.1, expected: 0.3 },
    { value: 0.3, min: 0, max: 0.3, step: 0.1, expected: 0.3 },
    { value: -0.85, min: -1, max: 1, step: 0.1, expected: -0.8 },
    { value: 2.5e-7, min: 1e-7, max: 6.5e-7, step: 1e-7, expected: 3e-7 },
    { value: 1e15, min: 0, max: 2e15, step: 1, expected: 1e15 },
  ];
  for (const { expected, ...props } of sliderCases) {
    const html = render(Slider, {
      props: { id: "stepped", label: "Stepped", name: "quantity", ...props },
    }).body;
    const inputs = [...html.matchAll(/<input[^>]*>/g)].map((match) => match[0]);
    assert.equal(inputs.length, 2);
    for (const input of inputs)
      assert.equal(
        input.match(/\bvalue="([^"]+)"/)[1],
        String(expected),
        JSON.stringify(props),
      );
    assert.equal(
      html.match(/<output[^>]*>(.*?)<\/output>/s)[1],
      String(expected),
      JSON.stringify(props),
    );
    assert.ok(inputs[0].includes('name="quantity"'));
  }
  const { default: Carousel } = await import(
    pathToFileURL(join(directory, "Carousel.js")).href
  );
  assert.throws(
    () =>
      render(Carousel, {
        props: {
          label: "Invalid",
          slides: [{ id: "one", label: "One" }],
          index: NaN,
        },
      }).body,
    /finite index/,
  );
  const date = await import(pathToFileURL(join(directory, "date.js")).href);
  assert.equal(date.addDays("2028-02-28", 1), "2028-02-29");
  assert.equal(date.addDays("2028-12-31", 1), "2029-01-01");
  assert.equal(date.addMonths("2028-01-31", 1), "2028-02-29");
  assert.throws(() => date.parseDate("2027-02-29"), /real calendar/);
  assert.throws(() => date.parseDate("2028-13-01"), /real calendar/);
  const { default: AspectRatio } = await import(
    pathToFileURL(join(directory, "AspectRatio.js")).href
  );
  assert.throws(
    () => render(AspectRatio, { props: { ratio: 0 } }).body,
    /positive finite/,
  );
  const { default: Progress } = await import(
    pathToFileURL(join(directory, "Progress.js")).href
  );
  assert.throws(
    () =>
      render(Progress, {
        props: { id: "bad", label: "Bad", value: 200, max: 100 },
      }).body,
    /within range/,
  );
  const matrix = JSON.parse(
    await readFile(join(root, "docs/components/matrix.json"), "utf8"),
  );
  assert.equal(matrix.length, 66);
  assert.equal(new Set(matrix.map((row) => row.name)).size, 66);
  const exports = await import(pathToFileURL(join(directory, "index.js")).href);
  for (const row of matrix) {
    assert.equal(typeof exports[row.export], "function", row.name);
    await readFile(join(root, row.spec));
    await readFile(join(root, row.reference));
  }
  console.log(
    "PASS 66 exports, 132 family/theme and 12 composed SSR cases, date boundaries, invalid input and component matrix",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
