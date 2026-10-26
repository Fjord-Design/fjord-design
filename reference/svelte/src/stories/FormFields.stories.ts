import type { Meta, StoryObj } from "@storybook/svelte-vite";
import FormFields from "./FormFields.svelte";

const meta = {
  title: "Components/Form fields",
  component: FormFields,
  args: { mode: "night", identity: "fjord" },
} satisfies Meta<typeof FormFields>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Night: Story = {};
export const Day: Story = { args: { mode: "day" } };
export const Aurora: Story = { args: { mode: "aurora" } };
export const Fern: Story = { args: { identity: "fern" } };
