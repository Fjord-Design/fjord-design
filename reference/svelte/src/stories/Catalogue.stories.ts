import type { Meta, StoryObj } from "@storybook/svelte-vite";
import Catalogue from "./Catalogue.svelte";
const meta = {
  title: "Fjord/Explore",
  component: Catalogue,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof Catalogue>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
