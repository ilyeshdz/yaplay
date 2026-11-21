import { copyFile, mkdir, writeFile } from 'node:fs/promises';
await mkdir(new URL('../public/', import.meta.url), { recursive: true });
await copyFile(new URL('../../examples/yaplay-client.mjs', import.meta.url), new URL('../public/yaplay-client.mjs', import.meta.url));
await writeFile(new URL('../public/.nojekyll', import.meta.url), '');
