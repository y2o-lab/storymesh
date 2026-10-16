import { readFileSync, writeFileSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const button = resolve(root, 'src/components/Button.stories.tsx');
const usage = resolve(root, 'src/components/UsageMeter.stories.tsx');
const loading = 'export const Loading: Story = { args: { loading: true, children: "プランを選択" } };\n';
const action = process.argv[2];
if (!['reset', 'complete'].includes(action)) throw new Error('Use reset or complete');
if (action === 'reset') {
  writeFileSync(button, readFileSync(button, 'utf8').replace(loading, ''));
  rmSync(usage, { force: true });
} else {
  const text = readFileSync(button, 'utf8');
  if (!text.includes('export const Loading:')) writeFileSync(button, text + loading);
  writeFileSync(usage, `import { UsageMeter as Component } from './UsageMeter';\n\nconst meta = {\n  title: 'Components/UsageMeter',\n  component: Component,\n};\n\nexport default meta;\n\nexport const Default = {};\n`);
}
console.log(`Demo scenario: ${action}. Rebuild the Storybook index before spec check.`);
