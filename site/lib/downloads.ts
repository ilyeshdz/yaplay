export const releaseUrl = 'https://github.com/ilyeshdz/yaplay/releases';
export const downloads = [
  { id: 'windows', label: 'Windows', detail: 'x86_64 · EXE', asset: 'yaplay-windows-x86_64.exe' },
  { id: 'mac-arm', label: 'macOS', detail: 'Apple Silicon · DMG', asset: 'Yaplay-aarch64-apple-darwin.dmg' },
  { id: 'mac-intel', label: 'macOS', detail: 'Intel · DMG', asset: 'Yaplay-x86_64-apple-darwin.dmg' },
] as const;
export function downloadUrl(asset: string) { return `${releaseUrl}/latest/download/${asset}`; }

export function detectOS(userAgent: string, platform: string, touchPoints = 0) {
  if (/Android|iPhone|iPad|iPod/i.test(userAgent) || (/Mac/i.test(platform) && touchPoints > 1)) return 'mobile';
  if (/Windows|Win32|Win64/i.test(`${userAgent} ${platform}`)) return 'windows';
  if (/Mac/i.test(`${userAgent} ${platform}`)) return 'mac';
  if (/Linux|X11/i.test(`${userAgent} ${platform}`)) return 'linux';
  return 'unknown';
}
