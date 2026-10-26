import type { Meta, StoryObj } from "@storybook/svelte-vite";
import SemanticModes from "./SemanticModes.svelte";

const meta = {
  title: "Foundations/Semantic modes",
  component: SemanticModes,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof SemanticModes>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const System: Story = { args: { mode: "system" } };
export const Fern: Story = { args: { identity: "fern" } };
