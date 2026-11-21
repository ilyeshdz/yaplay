// Only dated, versioned Release Please entries belong on the public release page.
export function parseReleases(changelog: string) {
  return changelog.split(/^## /m).flatMap((section) => {
    const [heading, ...lines] = section.split('\n');
    const match = heading.match(/^\[?(\d+\.\d+\.\d+)\]?(?:\([^)]+\))? \((\d{4}-\d{2}-\d{2})\)$/);
    if (!match) return [];
    return [{ version: match[1], date: match[2], body: lines.join('\n').replace(/<!--[\s\S]*?-->/g, '').trim() }];
  });
}
