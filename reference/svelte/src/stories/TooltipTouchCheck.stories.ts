import type { Meta, StoryObj } from "@storybook/svelte-vite";
import TooltipTouchCheck from "./TooltipTouchCheck.svelte";
const meta = {
  title: "Verification/Tooltip touch events",
  component: TooltipTouchCheck,
} satisfies Meta<typeof TooltipTouchCheck>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Events: Story = {};
