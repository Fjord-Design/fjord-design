import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const workspace = join(root, "reference/svelte");
const require = createRequire(join(workspace, "package.json"));
const compiler = join(
  dirname(require.resolve("@typescript/native/package.json")),
  "bin/tsc",
);
const { compilerOptions } = JSON.parse(
  await readFile(join(workspace, "tsconfig.json"), "utf8"),
);
const fixtures = join(workspace, ".verification");
await mkdir(fixtures, { recursive: true });
const directory = await mkdtemp(join(fixtures, "storybook-"));
const cases = [
  ["invalid-meta-args.ts", /TS(?:2322|2769)/],
  ["invalid-story-args.ts", /TS2769/],
  ["missing-story-args.ts", /TS2554/],
  ["invalid-mock-argument.ts", /TS2345/],
  ["invalid-svelte-args.ts", /TS2322/],
];
try {
  await cp(join(root, "tests/types/storybook"), directory, { recursive: true });
  const check = async (file) => {
    await writeFile(
      join(directory, "tsconfig.json"),
      JSON.stringify({ compilerOptions, files: [file] }),
    );
    const result = spawnSync(
      "node",
      [compiler, "--project", join(directory, "tsconfig.json")],
      { cwd: workspace, encoding: "utf8" },
    );
    if (result.error) throw result.error;
    return { ...result, output: result.stdout + result.stderr };
  };
  const valid = await check("valid.ts");
  assert.equal(valid.status, 0, valid.output);
  console.log(
    "PASS Storybook factories preserve defaults, required args, mock signatures and typed results",
  );
  for (const [file, diagnostic] of cases) {
    const invalid = await check(file);
    assert.equal(invalid.status, 1, `${file}\n${invalid.output}`);
    const messages = invalid.output
      .split("\n")
      .filter((line) => line.includes(`/${file}(`));
    assert.equal(messages.length, 1, invalid.output);
    assert.match(messages[0], diagnostic, invalid.output);
    assert.doesNotMatch(
      invalid.output,
      /Cannot find module|TS2578|chunk-.*\.d\.ts\(/,
    );
    console.log(`PASS rejected ${file}`);
  }
} finally {
  await rm(directory, { recursive: true, force: true });
}
