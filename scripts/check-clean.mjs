import { execFileSync } from "node:child_process";
import { cp, mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const directory = await mkdtemp(join(tmpdir(), "fjord-clean-"));
try {
  const files = execFileSync(
    "git",
    ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
    { cwd: root },
  )
    .toString()
    .split("\0")
    .filter(Boolean);
  for (const path of files) {
    const destination = join(directory, path);
    await mkdir(dirname(destination), { recursive: true });
    await cp(join(root, path), destination);
  }
  const env = { ...process.env, MISE_TRUSTED_CONFIG_PATHS: directory };
  const run = (args) =>
    execFileSync("mise", args, { cwd: directory, env, stdio: "inherit" });
  run(["run", "setup"]);
  run(["run", "ci"]);
  console.log(
    "PASS clean source copy installs and builds without existing generated files",
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
