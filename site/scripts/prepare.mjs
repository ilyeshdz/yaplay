import { copyFile, mkdir, writeFile } from 'node:fs/promises';
await mkdir(new URL('../public/', import.meta.url), { recursive: true });
await copyFile(new URL('../../examples/yaplay-client.mjs', import.meta.url), new URL('../public/yaplay-client.mjs', import.meta.url));
await writeFile(new URL('../public/.nojekyll', import.meta.url), '');
await mkdir(new URL('../public/branding/', import.meta.url), { recursive: true });
for (const file of ['logo.svg', 'wordmark.svg', 'banner.svg', 'banner.png', 'icon.svg', 'icon.ico', 'apple-touch-icon.png']) {
  await copyFile(new URL(`../../assets/branding/${file}`, import.meta.url), new URL(`../public/branding/${file}`, import.meta.url));
}
