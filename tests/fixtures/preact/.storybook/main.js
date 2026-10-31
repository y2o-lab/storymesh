export default {
  stories: ['../src/**/*.stories.@(js|jsx|ts|tsx)'],
  framework: '@storybook/preact-vite',
  async viteFinal(config) {
    config.esbuild = { ...config.esbuild, jsx: 'automatic', jsxImportSource: 'preact' };
    return config;
  },
};
