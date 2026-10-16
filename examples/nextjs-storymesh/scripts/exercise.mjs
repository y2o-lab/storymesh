import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, mkdirSync, rmSync, cpSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const out = resolve(root, '../../docs/blog/storymesh-nextjs/evidence');
const scratch = resolve(root, '.demo');
mkdirSync(out, { recursive: true });
mkdirSync(resolve(root, 'storybook-static'), { recursive: true });
rmSync(scratch, { recursive: true, force: true });
mkdirSync(scratch, { recursive: true });
const records = [];
let group = '01-install';
const shellArg = value => /^[A-Za-z0-9._/=-]+$/.test(value) ? value : JSON.stringify(value);
function run(args, expected = 0, cwd = root, command = `npx --no-install ${args.map(shellArg).join(' ')}`) {
 const r = spawnSync(resolve(root, 'node_modules/.bin', args[0]), args.slice(1), { cwd, encoding: 'utf8', env: { ...process.env, NO_COLOR: '1', FORCE_COLOR: '0', STORYBOOK_DISABLE_TELEMETRY: '1' } });
 if (r.error) throw r.error;
 records.push({ group, command, invocation: { executable: `node_modules/.bin/${args[0]}`, args: args.slice(1), cwd: cwd === root ? '.' : 'src/components' }, exitCode: r.status, stdout: r.stdout, stderr: r.stderr });
 writeFileSync(resolve(out, 'commands.json'), JSON.stringify(records, null, 2) + '\n');
 console.log(`$ ${command}\n${r.stdout}${r.stderr}[exit ${r.status}]`);
 if (r.status !== expected) throw new Error(`Expected ${expected}, got ${r.status}: ${command}`);
}
function scenario(action) {
 const r = spawnSync(process.execPath, ['scripts/scenario.mjs', action], { cwd: root, encoding: 'utf8' });
 if (r.status !== 0) throw new Error(r.stderr);
}
run(['storymesh', '--version']);
run(['storymesh', '--help']);
scenario('reset');
group = '02-missing';
run(['storymesh', 'check', 'src/components', '--framework', 'react'], 1);
run(['storymesh', 'coverage', 'src/components', '--framework', 'react']);
run(['storymesh', 'report', 'src/components', '--framework', 'react']);
group = '03-ignore';
run(['storymesh', 'check', 'src/components', '--ignore', 'UsageMeter.tsx']);
run(['storymesh', 'check', 'src/components', '--ignore-file', '.storymesh-manual-ignore']);
run(['storymesh', 'report', 'src/components', '--ignore', '!generated/PreviewCard.tsx']);
run(['storymesh', 'check'], 1, resolve(root, 'src/components'), 'cd src/components && npx --no-install storymesh check');
group = '04-generate';
run(['storymesh', 'check', 'src/components', '--framework', 'react', '--generate']);
writeFileSync(resolve(out, 'UsageMeter.generated.txt'), readFileSync(resolve(root, 'src/components/UsageMeter.stories.tsx'), 'utf8'));
run(['storymesh', 'check', 'src/components', '--generate']);
run(['storymesh', 'check', 'src/components']);
run(['storymesh', 'coverage', 'src/components']);
group = '05-spec-fail';
run(['storybook', 'index', '-o', 'storybook-static/index.json']);
cpSync(resolve(root, 'storybook-static/index.json'), resolve(out, 'index-before.json'));
run(['storymesh', 'spec', 'check'], 1);
group = '06-spec-pass';
scenario('complete');
run(['storybook', 'index', '-o', 'storybook-static/index.json']);
cpSync(resolve(root, 'storybook-static/index.json'), resolve(out, 'index-after.json'));
run(['storymesh', 'spec', 'check']);
group = '07-spec-init';
run(['storymesh', 'spec', 'init', 'src/components', '--framework', 'react', '--spec-dir', '.demo/drafts']);
run(['storymesh', 'spec', 'check', '--spec-dir', '.demo/drafts'], 2);
mkdirSync(resolve(scratch, 'fresh'), { recursive: true });
writeFileSync(resolve(scratch, 'fresh/FreshCard.tsx'), 'export function FreshCard() { return <div>Fresh card</div>; }\n');
run(['storymesh', 'spec', 'init', '.demo/fresh', '--framework', 'react', '--with-stories', '--spec-dir', '.demo/fresh-specs']);
cpSync(resolve(scratch, 'drafts'), resolve(out, 'drafts'), { recursive: true });
cpSync(resolve(scratch, 'fresh-specs'), resolve(out, 'fresh-specs'), { recursive: true });
cpSync(resolve(scratch, 'fresh/FreshCard.stories.tsx'), resolve(out, 'FreshCard.generated.txt'));
group = '08-spec-import';
run(['storymesh', 'spec', 'import', '--index', '../../docs/blog/storymesh-nextjs/evidence/index-before.json', '--spec-dir', '.demo/imported']);
run(['storymesh', 'spec', 'import', '--index', '../../docs/blog/storymesh-nextjs/evidence/index-before.json', '--spec-dir', '.demo/imported']);
cpSync(resolve(scratch, 'imported'), resolve(out, 'imported-original'), { recursive: true });
run(['storymesh', 'spec', 'import', '--spec-dir', '.demo/imported', '--merge'], 2);
// Public 0.1.0 emits indentless lists, but merge requires a two-space block list.
// Reformat only the generated list indentation, preserving the actual requirements.
for (const file of readdirSync(resolve(scratch, 'imported'))) {
 const path = resolve(scratch, 'imported', file);
 const text = readFileSync(path, 'utf8');
 const [header, list] = text.split('stories:\n');
 writeFileSync(path, header + 'stories:\n' + list.split('\n').map(line => line ? '  ' + line : line).join('\n'));
}
run(['storymesh', 'spec', 'import', '--spec-dir', '.demo/imported', '--merge']);
records.at(-1).preparation = '直前に生成 YAML の stories リストを 2 スペース下げた（値は変更なし）。';
run(['storymesh', 'spec', 'check', '--spec-dir', '.demo/imported']);
cpSync(resolve(scratch, 'imported'), resolve(out, 'imported'), { recursive: true });
group = '09-warning-error';
mkdirSync(resolve(scratch, 'warning'), { recursive: true });
writeFileSync(resolve(scratch, 'warning/Button.yaml'), readFileSync(resolve(root, '.storymesh/specs/Button.yaml'), 'utf8').replace('  - id: disabled\n    required: false\n', ''));
run(['storymesh', 'spec', 'check', '--spec-dir', '.demo/warning']);
run(['storymesh', 'spec', 'check', '--index', '.demo/does-not-exist.json'], 2);
for (const name of new Set(records.map(r => r.group))) writeFileSync(resolve(out, `${name}.txt`), records.filter(r => r.group === name).map(r => `$ ${r.command}\n${r.stdout}${r.stderr}[exit ${r.exitCode}]\n`).join('\n'));
console.log(`Verified ${records.length} commands.`);
