import type { Meta, StoryObj } from "@storybook/svelte-vite";
import FoundationIcons from "./FoundationIcons.svelte";

const meta = {
  title: "Foundations/Icons",
  component: FoundationIcons,
  args: { mode: "night" },
} satisfies Meta<typeof FoundationIcons>;

export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
