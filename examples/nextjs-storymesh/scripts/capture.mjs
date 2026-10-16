import { chromium } from 'playwright';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const docs = resolve(root, '../../docs/blog/storymesh-nextjs');
const images = resolve(docs, 'images');
mkdirSync(images, { recursive: true });
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 960 }, deviceScaleFactor: 1 });
const failures = [];
page.on('pageerror', error => failures.push(error.message));
const shots = [];
async function shot(name, description, url = page.url()) {
 await page.screenshot({ path: resolve(images, `${name}.png`), fullPage: true });
 shots.push({ file: `${name}.png`, description, url });
 console.log(`Captured ${name}`);
}
await page.goto('http://127.0.0.1:3000', { waitUntil: 'networkidle' });
await page.getByRole('heading', { name: 'チームの成長に、ちょうどいいプラン。' }).waitFor();
await shot('01-nextjs-app', 'Next.js アプリのプラン管理画面');
await page.getByRole('button', { name: '招待リンクを作成' }).click();
await page.getByRole('button', { name: '招待リンク作成済み' }).waitFor();
const progress = await page.getByRole('progressbar').getAttribute('aria-valuenow');
if (progress !== '80') throw new Error(`Expected 80% after invite, got ${progress}`);
await shot('02-nextjs-interaction', '招待操作後、利用人数が 7→8 名に更新');
async function story(id, expected, name, description) {
 await page.goto(`http://127.0.0.1:6006/?path=/story/${id}`, { waitUntil: 'networkidle' });
 const frame = page.frameLocator('#storybook-preview-iframe');
 await frame.getByText(expected, { exact: false }).first().waitFor({ timeout: 30000 });
 await page.screenshot({ path: resolve(images, `${name}.png`), fullPage: true });
 shots.push({ file: `${name}.png`, description, url: page.url() });
 console.log(`Captured ${name}`);
 return frame;
}
await story('components-plancard--default', 'チームのアイデアを、ひとつに。', '03-storybook-plancard', 'PlanCard の実描画と Controls');
const loading = await story('components-button--loading', '処理中…', '04-storybook-loading', 'required として追加した Loading Story');
if (!await loading.getByRole('button').isDisabled()) throw new Error('Loading must disable the button');
await story('components-usagemeter--default', 'メンバーの利用状況', '05-storybook-generated', 'Storymesh が生成した UsageMeter の Default を実描画');
await story('components-plancard--current', '現在のプラン', '06-storybook-current', 'Current Story の disabled UI');
const records = JSON.parse(readFileSync(resolve(docs, 'evidence/commands.json'), 'utf8'));
const escape = s => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
const strip = s => s.replace(/\x1b\[[0-9;]*m/g, '');
for (const group of new Set(records.map(r => r.group))) {
 const content = records.filter(r => r.group === group).map(r => `<section>${r.preparation ? `<p style="color:#a8bacb">${escape(r.preparation)}</p>` : ''}<div class="command">$ ${escape(r.command)}</div><pre>${escape(strip(r.stdout + r.stderr))}</pre><div class="exit">exit code: ${r.exitCode}</div></section>`).join('');
 const html = `<!doctype html><html lang="ja"><meta charset="utf-8"><title>Storymesh 実行ログ ${group}</title><style>*{box-sizing:border-box}body{margin:0;background:#0e1723;color:#dee8ee;padding:36px;font:16px/1.6 ui-monospace,Menlo,monospace}header{border-bottom:1px solid #304354;padding-bottom:20px;margin-bottom:24px}h1{font:700 24px Arial;margin:8px 0}header p{font:12px Arial;color:#94a5b7}section{margin:0 0 28px}.command{color:#8dd3b0;font-weight:700;overflow-wrap:anywhere}pre{margin:9px 0;white-space:pre-wrap;overflow-wrap:anywhere}.exit{color:#91a1b3;font-size:13px}</style><header><h1>Storymesh / ${group}</h1><p>実際の CLI 出力を保存したログビューアー · npm storymesh 0.1.0 · 出力は未編集（ANSI 色のみ除去）</p></header>${content}</html>`;
 writeFileSync(resolve(docs, `evidence/${group}.html`), html);
 await page.setViewportSize({ width: 1280, height: 900 });
 await page.setContent(html);
 await shot(`cli-${group}`, '保存した実際の CLI 実行結果のログビューアー', `evidence/${group}.html`);
}
if (failures.length) throw new Error(`Browser errors: ${failures.join('; ')}`);
writeFileSync(resolve(docs, 'evidence/screenshots.json'), JSON.stringify({ viewport: '1440×960 UI / 1280×900 logs', browser: await browser.version(), failures, shots, assertions: ['Next.js renders', 'Invite updates member usage to 80%', 'PlanCard Default renders', 'Loading renders and is disabled', 'Generated UsageMeter renders', 'Current renders'] }, null, 2) + '\n');
await browser.close();
