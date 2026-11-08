import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Complete from "./Complete.svelte";
const meta = {
  title: "Complete Library/GroupedForms",
  component: Complete,
  args: { family: "forms", mode: "night", identity: "fjord" },
} satisfies Meta<typeof Complete>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
