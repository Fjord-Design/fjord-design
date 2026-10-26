import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Layout from "./Layout.svelte";

const meta = {
  title: "Components/Card Separator Tabs",
  component: Layout,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Layout>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
