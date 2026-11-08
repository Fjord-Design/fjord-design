import type { Meta, StoryObj } from "@storybook/svelte-vite";
import ControlRegressions from "./ControlRegressions.svelte";

const meta = {
  title: "Verification/Control regressions",
  component: ControlRegressions,
} satisfies Meta<typeof ControlRegressions>;
export default meta;
type Story = StoryObj<typeof meta>;
export const SteppedValuesAndLongLists: Story = {};
