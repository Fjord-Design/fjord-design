import type { StoryObj } from "@storybook/svelte-vite";

export const story: StoryObj<{ mode: "night" | "day" | "aurora" }> = {
  args: { mode: "midnight" },
};
