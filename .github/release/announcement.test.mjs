import assert from 'node:assert/strict';
import test from 'node:test';
import { releaseMessage, sendAnnouncement } from './announce-release.mjs';
const role = '33367aaa-f65b-4e1c-a870-8f0041a437dc';
const release = {
  tag_name: 'v1.5.0',
  html_url: 'https://github.com/example/app/releases/tag/v1.5.0',
  body: '### Added\n\n- A useful update.',
  draft: false,
  prerelease: false,
};
test('compact branded embed has content, two link buttons, and a remote banner', () => {
  const p = releaseMessage(
    release,
    role,
    'https://raw.githubusercontent.com/example/app/sha/banner.png',
  );
  assert.ok(p.content.includes(`<@&${role}>`));
  assert.equal(p.embeds.length, 1);
  assert.equal(p.embeds[0].description, '- A useful update.');
  assert.ok(p.embeds[0].image.url.startsWith('https://raw.githubusercontent.com/'));
  assert.equal(p.components[0].components.length, 2);
  assert.ok(
    p.components[0].components.every(
      (b) => b.type === 2 && b.style === 5 && b.url.startsWith('https://'),
    ),
  );
  assert.deepEqual(p.allowed_mentions, { parse: [], roles: [role] });
  const long = releaseMessage({ ...release, body: '- A useful update.\n'.repeat(200) }, role);
  assert.ok(long.embeds[0].description.length <= 650);
  assert.ok(long.embeds[0].description.includes('full release notes'));
  assert.throws(() => releaseMessage({ ...release, prerelease: true }, role), /stable/);
  assert.throws(() => releaseMessage(release, role, 'file:///banner.png'), /HTTPS/);
});
test('delivery preserves its retry nonce without logging webhook credentials', async () => {
  const url = 'https://www.yapin.gg/api/webhooks/test/secret';
  await sendAnnouncement(
    release,
    url,
    role,
    async (target, options) => {
      assert.equal(target.searchParams.get('wait'), 'true');
      assert.equal(JSON.parse(options.body).nonce, 'release-12345678');
      return { ok: true };
    },
    undefined,
    'release-12345678',
  );
  await assert.rejects(
    sendAnnouncement(release, url, role, async () => {
      throw new Error(url);
    }),
    (e) => !e.message.includes('secret'),
  );
});
