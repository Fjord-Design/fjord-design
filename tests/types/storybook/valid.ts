import { __definePreview, type ReactTypes } from "@storybook/react";
import type { StoryObj } from "@storybook/svelte-vite";

type Props = {
  label: string;
  count?: number;
  onSelect: (id: number) => void;
};

export const Button = (_props: Props) => null;
const selectMock = () => {
  const calls: [number][] = [];
  return Object.assign(
    (id: number) => {
      calls.push([id]);
    },
    { mock: { calls } },
  );
};
export const preview = __definePreview({ addons: [] });
export const meta = preview.meta({
  component: Button,
  args: { label: "Fjord", onSelect: selectMock() },
});
export const incomplete = preview.meta({
  component: Button,
  args: { label: "Fjord" },
});

export const story = meta.story({
  args: { count: 2 },
  play: ({ args }) => {
    args.onSelect(1);
    const call: number | undefined = args.onSelect.mock.calls[0]?.[0];
    const label: string = args.label;
    void [call, label];
  },
});

const supplied = incomplete.story({
  args: { onSelect: selectMock() },
});
const defaultStory = meta.story();
declare const render: () => ReactTypes["storyResult"];
const rendered = meta.story(render);
const label: string = story.composed.args.label;
const count: number = story.input.args.count;
const svelteStory: StoryObj<{ mode: "night" | "day" | "aurora" }> = {
  args: { mode: "day" },
};
void [supplied, defaultStory, rendered, label, count, svelteStory];
