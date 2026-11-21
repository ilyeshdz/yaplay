import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../out/', import.meta.url));
const base = process.env.NEXT_PUBLIC_BASE_PATH || '';
async function files(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  return (await Promise.all(entries.map((entry) => {
    const name = path.join(dir, entry.name);
    return entry.isDirectory() ? files(name) : [name];
  }))).flat();
}
const htmlFiles = (await files(root)).filter((file) => file.endsWith('.html'));
let checked = 0;
for (const file of htmlFiles) {
  const html = await readFile(file, 'utf8');
  const relative = path.relative(root, file).split(path.sep).join('/');
  const pageUrl = new URL(`${base}/${relative}`, 'https://static.example');
  for (const [, attribute] of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
    const url = new URL(attribute.replaceAll('&amp;', '&'), pageUrl);
    if (url.origin !== pageUrl.origin) continue;
    if (base && !url.pathname.startsWith(`${base}/`)) {
      throw new Error(`${relative}: link escapes deployment path: ${attribute}`);
    }
    const pathname = decodeURIComponent(url.pathname.slice(base.length));
    let target = path.join(root, pathname);
    let info;
    try { info = await stat(target); } catch {
      throw new Error(`${relative}: missing export for ${attribute}`);
    }
    if (info.isDirectory()) target = path.join(target, 'index.html');
    const body = await readFile(target);
    if (url.hash && target.endsWith('.html')) {
      const id = decodeURIComponent(url.hash.slice(1));
      if (!body.toString().includes(`id="${id}"`)) {
        throw new Error(`${relative}: missing anchor ${attribute}`);
      }
    }
    checked++;
  }
}
await stat(path.join(root, 'docs', 'protocol', 'index.html'));
await stat(path.join(root, 'yaplay-client.mjs'));
await stat(path.join(root, '.nojekyll'));
console.log(`Validated ${htmlFiles.length} static pages and ${checked} local links/assets (base: ${base || '/'}).`);
