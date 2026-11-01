import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fixtures, selectFixtures } from './fixture-plan.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const args = process.argv.slice(2);
function run(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', env: {
    ...process.env, CI: 'true', STORYBOOK_DISABLE_TELEMETRY: '1',
  } });
  if (result.error || result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed: ${result.error?.message ?? result.status}`);
  }
}
function git(args) {
  const result = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(result.stderr || result.error?.message);
  return result.stdout.split('\0').filter(Boolean);
}
function selectedFixtures() {
  if (args.length === 0) return Object.keys(fixtures);
  if (args.length !== 1 || args[0] !== '--changed') throw new Error('Usage: node scripts/check-fixture-builds.mjs [--changed]');
  const base = process.env.STORYMESH_FIXTURE_BASE || 'HEAD';
  // Includes committed PR changes, staged/unstaged changes and new files locally.
  const changed = [...git(['diff', '--name-only', '-z', base, '--']),
    ...git(['ls-files', '--others', '--exclude-standard', '-z'])];
  return selectFixtures(changed);
}

try {
  const selected = selectedFixtures();
  if (!selected.length) {
    console.log('No guarded fixture or generation changes; no Storybook builds selected.');
  } else {
    console.log(`Storybook fixture builds: ${selected.join(', ')}`);
    run('mise', ['exec', '--', 'cargo', 'build', '--locked']);
  }
  for (const name of selected) {
    const source = join(root, 'tests/fixtures', name);
    if (!existsSync(join(source, 'node_modules/.bin/storybook'))) {
      if (process.env.STORYMESH_FIXTURE_INSTALL === '1') run('mise', ['exec', '--', 'npm', 'ci'], source);
    }
    if (!existsSync(join(source, 'node_modules/.bin/storybook'))) {
      throw new Error(`Missing ${name} dependencies. Run: mise exec -- npm ci --prefix tests/fixtures/${name}`);
    }
    const temporary = mkdtempSync(join(tmpdir(), `storymesh-${name}-`));
    try {
      cpSync(source, temporary, { recursive: true, filter: path => {
        const entry = basename(path);
        return !['node_modules', 'storybook-static', '.git', '.cache'].includes(entry)
          && !/\.stories\.[^.]+$/.test(entry);
      } });
      symlinkSync(resolve(source, 'node_modules'), join(temporary, 'node_modules'), 'junction');
      run(join(resolve(root, process.env.CARGO_TARGET_DIR || 'target'), `debug/storymesh${process.platform === 'win32' ? '.exe' : ''}`), ['check', join(temporary, 'src'), '--framework', fixtures[name], '--generate']);
      run('mise', ['exec', '--', 'npm', 'run', 'build-storybook', '--', '--output-dir', 'storybook-static'], temporary);
      const entries = Object.values(JSON.parse(readFileSync(join(temporary, 'storybook-static/index.json'), 'utf8')).entries);
      let storyCount = 0;
      function verifyStories(directory) {
        for (const entry of readdirSync(directory, { withFileTypes: true })) {
          const path = join(directory, entry.name);
          if (entry.isDirectory()) verifyStories(path);
          else if (/\.stories\.[^.]+$/.test(entry.name)) {
            storyCount++;
            const importPath = `./${relative(temporary, path).split('\\').join('/')}`;
            if (!entries.some(story => story.type === 'story' && story.importPath === importPath && story.id.endsWith('--default'))) {
              throw new Error(`${name}: generated story ${path} has no Default entry in the built Storybook index`);
            }
          }
        }
      }
      verifyStories(join(temporary, 'src'));
      if (storyCount === 0) throw new Error(`${name}: no generated stories were checked`);
      console.log(`Verified ${name}: generated stories build and register Default entries.`);
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
