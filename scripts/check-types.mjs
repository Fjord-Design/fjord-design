import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { access, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const workspace = join(root, "reference/svelte");
const manifest = JSON.parse(
  await readFile(join(workspace, "package.json"), "utf8"),
);
const versions = manifest.devDependencies;
const run = (command, args, cwd = workspace) => {
  const result = spawnSync(command, args, { cwd, encoding: "utf8" });
  if (result.error) throw result.error;
  return { ...result, output: result.stdout + result.stderr };
};
const checkArgs = [
  "run",
  "svelte-check",
  "--tsgo",
  "--fail-on-warnings",
  "--tsconfig",
  "./tsconfig.library.json",
];
const consumerCheckArgs = [...checkArgs.slice(0, -1), "./tsconfig.json"];
const sourceCheck = () =>
  run("node", [
    join(root, "scripts/check-reference.mjs"),
    "tsconfig.library.json",
  ]);
const nativeCheck = (args, cwd = workspace) => {
  const require = createRequire(join(cwd, "package.json"));
  const compilerPackage = require.resolve("@typescript/native/package.json");
  return run("node", [join(dirname(compilerPackage), "bin/tsc"), ...args], cwd);
};
const expectValid = (result, label) => {
  assert.equal(result.status, 0, `${label}\n${result.output}`);
  console.log(`PASS ${label}`);
};
const expectDiagnostic = (result, file, label) => {
  assert.equal(
    result.status,
    1,
    `${label}: expected a type error\n${result.output}`,
  );
  assert.match(result.output, new RegExp(file.replaceAll(".", "\\.")));
  assert.match(result.output, /not assignable/);
  assert.doesNotMatch(
    result.output,
    /Cannot find module|FAILURE|command not found/,
  );
  console.log(`PASS ${label}`);
};

const compiler = nativeCheck(["--version"]);
expectValid(compiler, "native compiler runs");
assert.match(compiler.output, /Version 7\./);
expectValid(sourceCheck(), "valid Svelte library source");

const probes = await mkdtemp(join(workspace, "src/lib/type-verification-"));
const probe = join(probes, "plain.ts");
const propProbe = join(probes, "PropVerification.svelte");
const plainConfig = join(probes, "tsconfig.json");
const consumer = await mkdtemp(join(tmpdir(), "fjord-consumer-"));
try {
  await writeFile(
    plainConfig,
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
      },
      files: ["plain.ts"],
    }),
  );
  await writeFile(probe, "export const count: number = 1;\n", { flag: "wx" });
  expectValid(
    nativeCheck(["--project", plainConfig]),
    "valid plain TypeScript source",
  );
  await writeFile(probe, 'export const count: number = "fjord";\n', {
    flag: "w",
  });
  expectDiagnostic(
    nativeCheck(["--project", plainConfig]),
    "plain.ts",
    "plain TypeScript error rejected by TS7",
  );
  await rm(probe);

  await writeFile(
    propProbe,
    '<script lang="ts">import { ThemeScope } from "../index.js";</script>\n<ThemeScope mode="midnight" />\n',
    { flag: "wx" },
  );
  expectDiagnostic(
    sourceCheck(),
    "PropVerification.svelte",
    "invalid Svelte prop rejected by TS7",
  );
  await rm(propProbe);
  await writeFile(
    propProbe,
    '<script lang="ts">import { Button } from "../index.js";</script>\n<Button label="Search" iconOnly />\n',
  );
  expectDiagnostic(
    sourceCheck(),
    "PropVerification.svelte",
    "icon-only button without icon rejected",
  );
  await rm(propProbe);
  await writeFile(
    propProbe,
    '<script lang="ts">import { Input } from "../index.js";</script>\n<Input type="checkbox" />\n',
  );
  expectDiagnostic(
    sourceCheck(),
    "PropVerification.svelte",
    "unsupported input type rejected",
  );
  await rm(propProbe);
  await writeFile(
    propProbe,
    '<script lang="ts">import { Icon } from "../index.js";</script>\n<Icon name="info" decorative={false} />\n',
  );
  expectDiagnostic(
    sourceCheck(),
    "PropVerification.svelte",
    "meaningful icon without a label rejected",
  );
  await rm(propProbe);
  expectValid(
    sourceCheck(),
    "workspace passes after removing invalid fixtures",
  );

  const artifacts = join(root, "dist");
  const packed = run(
    "bun",
    ["pm", "pack", "--destination", artifacts],
    join(root, "dist/svelte"),
  );
  expectValid(packed, "local library archive built");
  const archive = join(artifacts, `fjord-svelte-${manifest.version}.tgz`);
  await writeFile(
    join(consumer, "package.json"),
    JSON.stringify(
      {
        name: "fjord-independent-consumer",
        private: true,
        type: "module",
        dependencies: {
          "@fjord/svelte": `file:${archive}`,
          svelte: versions.svelte,
        },
        devDependencies: {
          "@typescript/native": versions["@typescript/native"],
          typescript: versions.typescript,
          "svelte-check": versions["svelte-check"],
        },
      },
      null,
      2,
    ),
  );
  await writeFile(
    join(consumer, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          strict: true,
          noEmit: true,
          lib: ["ES2022", "DOM"],
        },
        include: ["*.ts", "*.svelte"],
      },
      null,
      2,
    ),
  );
  expectValid(
    run("bun", ["install", "--prefer-offline"], consumer),
    "independent consumer installed from archive and pinned dependencies",
  );
  const consumerLock = await readFile(join(consumer, "bun.lock"));
  expectValid(
    run("bun", ["install", "--offline", "--frozen-lockfile"], consumer),
    "consumer lockfile install",
  );
  assert.deepEqual(await readFile(join(consumer, "bun.lock")), consumerLock);
  const installed = JSON.parse(
    await readFile(
      join(consumer, "node_modules/@fjord/svelte/package.json"),
      "utf8",
    ),
  );
  assert.equal(installed.exports["."].types, "./index.d.ts");
  assert.equal(installed.exports["./foundations.css"], "./foundations.css");
  assert.equal(installed.exports["./semantics.css"], "./semantics.css");
  assert.equal(installed.exports["./form-controls.css"], "./form-controls.css");
  assert.equal(installed.exports["./choices.css"], "./choices.css");
  assert.equal(
    installed.exports["./layout-components.css"],
    "./layout-components.css",
  );
  assert.equal(
    installed.exports["./overlay-components.css"],
    "./overlay-components.css",
  );
  assert.equal(
    installed.exports["./type-components.css"],
    "./type-components.css",
  );
  await access(
    join(consumer, "node_modules/@fjord/svelte/type-components.css"),
  );
  assert.equal(installed.exports["./data-feedback.css"], "./data-feedback.css");
  const dataCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/data-feedback.css"),
    "utf8",
  );
  assert.match(dataCss, /prefers-reduced-motion/);
  const overlayCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/overlay-components.css"),
    "utf8",
  );
  assert.match(overlayCss, /forced-colors/);
  const layoutCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/layout-components.css"),
    "utf8",
  );
  assert.match(layoutCss, /forced-colors/);
  const choiceCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/choices.css"),
    "utf8",
  );
  assert.match(choiceCss, /forced-colors/);
  const semanticCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/semantics.css"),
    "utf8",
  );
  assert.match(semanticCss, /data-fjord-identity="fern"/);
  assert.match(semanticCss, /prefers-color-scheme:light/);
  const formCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/form-controls.css"),
    "utf8",
  );
  assert.match(formCss, /textarea\.fjord-form-control/);
  assert.match(formCss, /forced-colors/);
  const iconRoot = join(consumer, "node_modules/@fjord/svelte/icons");
  const iconManifest = JSON.parse(
    await readFile(join(iconRoot, "manifest.json"), "utf8"),
  );
  for (const asset of [
    { file: "LICENSE", sha256: iconManifest.licenseSha256 },
    ...iconManifest.icons,
  ]) {
    assert.equal(
      createHash("sha256")
        .update(await readFile(join(iconRoot, asset.file)))
        .digest("hex"),
      asset.sha256,
    );
  }
  const foundationCss = await readFile(
    join(consumer, "node_modules/@fjord/svelte/foundations.css"),
    "utf8",
  );
  assert.match(foundationCss, /prefers-reduced-motion:reduce/);
  assert.match(foundationCss, /forced-colors:active/);
  console.log(
    "PASS consumer receives foundation CSS, pinned icons and original MIT notice",
  );
  assert.equal(installed.exports["./typography.css"], "./fonts/typography.css");
  const fontRoot = join(consumer, "node_modules/@fjord/svelte/fonts");
  const fontManifest = JSON.parse(
    await readFile(join(fontRoot, "manifest.json"), "utf8"),
  );
  const typography = await readFile(join(fontRoot, "typography.css"), "utf8");
  const fontUrls = [...typography.matchAll(/url\("\.\/(.*?)"\)/g)].map(
    (match) => match[1],
  );
  assert.equal(fontUrls.length, 9);
  for (const family of fontManifest.families) {
    for (const asset of [
      { file: family.licenseFile, sha256: family.licenseSha256 },
      ...family.faces,
    ]) {
      const bytes = await readFile(join(fontRoot, asset.file));
      assert.equal(
        createHash("sha256").update(bytes).digest("hex"),
        asset.sha256,
      );
      if (asset.file.endsWith(".woff2"))
        assert.ok(fontUrls.includes(asset.file));
    }
  }
  console.log(
    "PASS consumer archive includes stylesheet, nine pinned fonts and five original licences",
  );
  const declaration = await readFile(
    join(consumer, "node_modules/@fjord/svelte/ThemeScope.svelte.d.ts"),
    "utf8",
  );
  assert.match(declaration, /Mode/);

  const component = (mode) =>
    `<script lang="ts">import { ThemeScope, Icon, Button, Badge, Field, Input, Textarea, NativeSelect, Checkbox, RadioGroup, Switch, Card, Separator, Tabs, Dialog, Alert, Tooltip, Table, Empty, Skeleton, Spinner, Typography, CodeBlock, Kbd } from "@fjord/svelte";</script>\n<ThemeScope mode="${mode}"><p>Consumer content</p><Icon name="info" decorative={false} label="Information" /><Button label="Search" iconOnly icon="search" /><Badge label="Ready" variant="success" /><Field id="address" label="Address">{#snippet children(control)}<Input {...control} name="address" autocomplete="street-address" />{/snippet}</Field><Textarea aria-label="Note" /><NativeSelect aria-label="Mode"><option value="night">Night</option></NativeSelect><Checkbox label="Accept" /><Switch label="Updates" /><RadioGroup id="freq" name="freq" label="Frequency" options={[{value:"day",label:"Daily"}]} /><Card><Separator /><Tabs id="tabs" label="Sections" items={[{value:"a",label:"First"}]}>{#snippet children(panel)}<p>{panel}</p>{/snippet}</Tabs></Card><Dialog id="review" title="Review" closeLabel="Close"><p>Body</p></Dialog><Alert title="Info" message="Ready" /><Tooltip id="hint" text="More">{#snippet children(attrs)}<Button {...attrs} label="Help" />{/snippet}</Tooltip><Table id="results" caption="Results"><thead><tr><th scope="col">Name</th></tr></thead><tbody><tr><td>Fjord</td></tr></tbody></Table><Empty title="No results" description="Try again" /><Skeleton /><Spinner label="Loading" /><Typography as="h2" role="title-2">Title</Typography><CodeBlock label="Example" code="const x = 1;" /><Kbd>Ctrl</Kbd></ThemeScope>\n`;
  await writeFile(join(consumer, "Consumer.svelte"), component("day"));
  await writeFile(
    join(consumer, "consumer.ts"),
    'import type { Mode } from "@fjord/svelte";\nexport const mode: Mode = "aurora";\n',
  );
  expectValid(
    run("bun", consumerCheckArgs, consumer),
    "valid consumer uses distributed declarations",
  );
  await writeFile(join(consumer, "Consumer.svelte"), component("system"));
  expectValid(
    run("bun", consumerCheckArgs, consumer),
    "consumer accepts the explicit system preference",
  );
  await writeFile(
    join(consumer, "Consumer.svelte"),
    component("day").replace(' label="Information"', ""),
  );
  expectDiagnostic(
    run("bun", consumerCheckArgs, consumer),
    "Consumer.svelte",
    "consumer meaningful icon without a label rejected",
  );
  await writeFile(join(consumer, "Consumer.svelte"), component("midnight"));
  expectDiagnostic(
    run("bun", consumerCheckArgs, consumer),
    "Consumer.svelte",
    "consumer Svelte prop error rejected",
  );
  await writeFile(join(consumer, "Consumer.svelte"), component("day"));
  await writeFile(
    join(consumer, "consumer.ts"),
    'import type { Mode } from "@fjord/svelte";\nexport const mode: Mode = "midnight";\n',
  );
  expectDiagnostic(
    nativeCheck(["--project", "tsconfig.json"], consumer),
    "consumer.ts",
    "consumer TypeScript error rejected",
  );
  await writeFile(
    join(consumer, "consumer.ts"),
    'import type { Mode } from "@fjord/svelte";\nexport const mode: Mode = "night";\n',
  );
  expectValid(
    run("bun", consumerCheckArgs, consumer),
    "corrected consumer passes",
  );
} finally {
  await rm(probes, { force: true, recursive: true });
  await rm(consumer, { force: true, recursive: true });
}
