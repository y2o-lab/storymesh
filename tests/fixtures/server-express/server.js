import express from 'express';
import { readFile } from 'node:fs/promises';
const app = express();
app.use((req, res, next) => { res.set('Access-Control-Allow-Origin', '*'); next(); });
app.get('/storybook_preview/button', async (_req, res) => res.type('html').send(await readFile(new URL('./components/button.html', import.meta.url), 'utf8')));
app.listen(8771, '127.0.0.1');
