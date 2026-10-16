import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { PlanCard } from "./PlanCard";
const meta = { title: "Components/PlanCard", component: PlanCard, tags: ["autodocs"] } satisfies Meta<typeof PlanCard>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Default: Story = {};
export const Current: Story = { args: { current: true } };
