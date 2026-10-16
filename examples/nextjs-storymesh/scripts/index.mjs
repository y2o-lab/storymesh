import { mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
mkdirSync('storybook-static', { recursive: true });
const result = spawnSync('storybook', ['index', '-o', 'storybook-static/index.json'], { stdio: 'inherit' });
if (result.error) throw result.error;
process.exit(result.status ?? 1);
