import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Overlays from "./Overlays.svelte";

const meta = {
  title: "Components/Dialog Alert Tooltip",
  component: Overlays,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Overlays>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
