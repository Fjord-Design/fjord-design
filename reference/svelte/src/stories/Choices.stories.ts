import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Choices from "./Choices.svelte";

const meta = {
  title: "Components/Choice controls",
  component: Choices,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Choices>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
