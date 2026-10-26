import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Environment from "./Environment.svelte";

const meta = {
  title: "Setup/Environment",
  component: Environment,
  args: { mode: "night" },
} satisfies Meta<typeof Environment>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
