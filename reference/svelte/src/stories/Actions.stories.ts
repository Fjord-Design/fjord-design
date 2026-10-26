import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Actions from "./Actions.svelte";

const meta = {
  title: "Components/Button and Badge",
  component: Actions,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Actions>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
