import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Compositions from "./Compositions.svelte";
const meta = {
  title: "Complete Library/Compositions",
  component: Compositions,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Compositions>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
export const Work: Story = { args: { identity: "work" } };
export const Gaming: Story = { args: { identity: "gaming" } };
