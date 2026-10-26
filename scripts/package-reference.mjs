import { copyFile, cp, mkdir, readFile, writeFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const source = JSON.parse(
  await readFile(new URL("reference/svelte/package.json", root), "utf8"),
);
const destination = new URL("dist/svelte/", root);
await mkdir(destination, { recursive: true });
await writeFile(
  new URL("package.json", destination),
  JSON.stringify(
    {
      name: source.name,
      version: source.version,
      type: "module",
      license: source.license,
      exports: {
        ".": {
          types: "./index.d.ts",
          svelte: "./index.js",
          default: "./index.js",
        },
        "./typography.css": "./fonts/typography.css",
        "./foundations.css": "./foundations.css",
        "./semantics.css": "./semantics.css",
        "./form-controls.css": "./form-controls.css",
        "./choices.css": "./choices.css",
        "./layout-components.css": "./layout-components.css",
        "./overlay-components.css": "./overlay-components.css",
      },
      files: [
        "*.js",
        "*.ts",
        "*.svelte",
        "*.css",
        "fonts",
        "foundations.css",
        "semantics.css",
        "icons",
        "LICENSE",
        "README.md",
      ],
      peerDependencies: source.peerDependencies,
    },
    null,
    2,
  ) + "\n",
);
await copyFile(new URL("LICENSE", root), new URL("LICENSE", destination));
await copyFile(
  new URL("dist/semantics.css", root),
  new URL("semantics.css", destination),
);
await copyFile(
  new URL("dist/foundations.css", root),
  new URL("foundations.css", destination),
);
await cp(new URL("dist/icons/", root), new URL("icons/", destination), {
  recursive: true,
});
await cp(new URL("dist/fonts/", root), new URL("fonts/", destination), {
  recursive: true,
});
await copyFile(
  new URL("reference/svelte/README.md", root),
  new URL("README.md", destination),
);
