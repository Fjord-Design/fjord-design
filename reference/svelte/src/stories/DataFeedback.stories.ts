import type { Meta, StoryObj } from "@storybook/svelte-vite";
import DataFeedback from "./DataFeedback.svelte";
const meta = {
  title: "Components/Table Empty Skeleton Spinner",
  component: DataFeedback,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof DataFeedback>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
