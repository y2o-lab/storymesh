import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { Button } from "./Button";
const meta = { title: "Components/Button", component: Button, tags: ["autodocs"] } satisfies Meta<typeof Button>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Default: Story = { args: { children: "プランを選択" } };
export const Disabled: Story = { args: { disabled: true, children: "現在のプラン" } };
export const Loading: Story = { args: { loading: true, children: "プランを選択" } };
