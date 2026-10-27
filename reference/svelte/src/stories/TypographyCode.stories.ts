import type { Meta, StoryObj } from "@storybook/svelte-vite";
import TypographyCode from "./TypographyCode.svelte";
const meta = {
  title: "Components/Typography Code Block Kbd",
  component: TypographyCode,
  parameters: { layout: "fullscreen" },
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof TypographyCode>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
