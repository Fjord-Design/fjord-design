import { spawnSync } from "node:child_process";
import { rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const workspace = fileURLToPath(
  new URL("../reference/svelte/", import.meta.url),
);
const config = process.argv[2] ?? "tsconfig.json";
if (!["tsconfig.json", "tsconfig.library.json"].includes(config)) {
  throw new Error("Choose tsconfig.json or tsconfig.library.json.");
}
// svelte-check 4.7.7 retains deleted virtual files without --incremental.
// Recreate its generated overlays so diagnostics describe the current sources.
await rm(join(workspace, ".svelte-check"), { recursive: true, force: true });
await rm(join(workspace, ".svelte-kit/.svelte-check"), {
  recursive: true,
  force: true,
});
const result = spawnSync(
  "bun",
  ["run", "svelte-check", "--tsgo", "--fail-on-warnings", "--tsconfig", config],
  {
    cwd: workspace,
    stdio: "inherit",
  },
);
if (result.error) throw result.error;
process.exit(result.status ?? 1);
