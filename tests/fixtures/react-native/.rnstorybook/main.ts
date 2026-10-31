import type { StorybookConfig } from '@storybook/react-native';
const config: StorybookConfig = { stories: ['../components/**/*.stories.@(js|jsx|ts|tsx)'], deviceAddons: [] };
export default config;
