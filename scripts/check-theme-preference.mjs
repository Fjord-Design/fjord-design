import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFile, writeFile, mkdir, mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { watchMode } from "../reference/svelte/src/lib/theme-preference.js";

const listeners = new Set();
const query = {
  matches: true,
  addEventListener(type, fn) {
    assert.equal(type, "change");
    listeners.add(fn);
  },
  removeEventListener(type, fn) {
    assert.equal(type, "change");
    listeners.delete(fn);
  },
};
const seen = [];
const record = (mode) => seen.push(mode);
watchMode("night", record)();
watchMode("system", record)();
assert.deepEqual(seen, ["night", "night"]);
assert.throws(() => watchMode("midnight", record), /Choose night/);
globalThis.window = {
  matchMedia(text) {
    assert.equal(text, "(prefers-color-scheme: light)");
    return query;
  },
};
try {
  seen.length = 0;
  const stop = watchMode("system", record);
  assert.deepEqual(seen, ["day"]);
  assert.equal(listeners.size, 1);
  query.matches = false;
  for (const listener of listeners) listener();
  assert.deepEqual(seen, ["day", "night"]);
  stop();
  assert.equal(listeners.size, 0);
  for (const mode of ["night", "day", "aurora"]) {
    const before = seen.length;
    watchMode(mode, record)();
    assert.equal(seen.length, before + 1);
    assert.equal(seen.at(-1), mode);
    assert.equal(listeners.size, 0);
  }
} finally {
  delete globalThis.window;
}
console.log(
  "PASS explicit modes, SSR dark default, system changes and listener cleanup",
);

const root = fileURLToPath(new URL("../", import.meta.url));
const workspace = join(root, "reference/svelte");
const require = createRequire(join(workspace, "package.json"));
const { compile } = require("svelte/compiler");
const { render } = await import(
  pathToFileURL(require.resolve("svelte/server")).href
);
await mkdir(join(workspace, ".verification"), { recursive: true });
const directory = await mkdtemp(join(workspace, ".verification/system-ssr-"));
try {
  const source = await readFile(
    join(workspace, "src/lib/ThemeScope.svelte"),
    "utf8",
  );
  await writeFile(
    join(directory, "ThemeScope.js"),
    compile(source, { generate: "server", filename: "ThemeScope.svelte" }).js
      .code,
  );
  await writeFile(
    join(directory, "theme-preference.js"),
    await readFile(join(workspace, "src/lib/theme-preference.js")),
  );
  const fixture =
    '<script>import ThemeScope from "./ThemeScope.js";</script><ThemeScope mode="system" identity="fern"><section class="fjord-foundation fjord-semantic fjord-type-body"><div class="fjord-page"><h1>System mode without JavaScript</h1><p>The system preference applies through CSS before hydration.</p><button class="fjord-control semantic-primary">Server-rendered action</button></div></section></ThemeScope>';
  await writeFile(
    join(directory, "Fixture.js"),
    compile(fixture, { generate: "server", filename: "Fixture.svelte" }).js
      .code,
  );
  const { default: Fixture } = await import(
    pathToFileURL(join(directory, "Fixture.js")).href
  );
  const result = render(Fixture);
  assert.match(result.body, /data-fjord-mode="night"/);
  assert.match(result.body, /data-fjord-preference="system"/);
  assert.match(result.body, /data-fjord-identity="fern"/);
  assert.doesNotMatch(result.body, /<script/);
  await mkdir(join(root, "dist/verification"), { recursive: true });
  const html =
    '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link rel="stylesheet" href="../fonts/typography.css"><link rel="stylesheet" href="../foundations.css"><link rel="stylesheet" href="../semantics.css"><title>Fjord system SSR</title></head><body style="margin:0">' +
    result.body +
    "</body></html>";
  await writeFile(
    join(root, "dist/verification/xer-107-system-ssr.html"),
    html,
  );
  console.log(
    "PASS Svelte server rendering uses a stable dark mode and opt-in system CSS without JavaScript",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
