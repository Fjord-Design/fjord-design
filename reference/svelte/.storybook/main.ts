import type { StorybookConfig } from "@storybook/svelte-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.ts"],
  framework: "@storybook/svelte-vite",
  core: { disableTelemetry: true },
};

export default config;
