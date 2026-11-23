import { readFile } from 'node:fs/promises';
import path from 'node:path';
import Link from 'next/link';
import Markdown from 'react-markdown';
import { Wordmark } from '@/components/wordmark';
import { parseReleases } from '@/lib/releases';
import { releaseUrl } from '@/lib/downloads';

export const metadata = { title: 'Release notes', description: 'What’s new in Yaplay. Features, fixes, and improvements in every release.' };

export default async function Releases() {
  const releases = parseReleases(await readFile(path.join(process.cwd(), '..', 'CHANGELOG.md'), 'utf8'));
  return (
    <div className="home">
      <header className="home-nav home-wrap">
        <Link href="/" aria-label="Yaplay home"><Wordmark /></Link>
        <nav aria-label="Main navigation"><Link href="/#download">Download</Link><Link href="/docs">Docs</Link><Link href="/releases" aria-current="page">Releases</Link></nav>
      </header>
      <main className="release-page home-wrap" id="main">
        <h1>Release notes</h1>
        <p className="release-intro">What’s new, what’s improved, and what’s fixed in Yaplay.</p>
        <nav className="release-index" aria-label="Release versions">{releases.map(({ version }) => <a key={version} href={`#v${version}`}>v{version}</a>)}</nav>
        {releases.map(({ version, date, body }, index) => (
          <article className="release-entry" id={`v${version}`} key={version}>
            <header><h2>v{version}</h2>{index === 0 && <span className="release-badge">Latest</span>}<time dateTime={date}>{date}</time></header>
            <div className="prose release-body">
              <Markdown components={{
                a: ({ href, children }) => {
                  const doc = href?.match(/^site\/content\/docs\/(.+?)\.mdx([?#].*)?$/);
                  if (doc) {
                    const slug = doc[1] === 'index' ? '' : doc[1].replace(/\/index$/, '');
                    return <Link href={`/docs/${slug}${doc[2] ?? ''}`}>{children}</Link>;
                  }
                  return <a href={href}>{children}</a>;
                },
              }}>{body}</Markdown>
            </div>
            <a className="home-link" href={`${releaseUrl}/tag/v${version}`}>View release and downloads on GitHub ↗</a>
          </article>
        ))}
        <p className="release-source">These notes come from the changelog maintained by Release Please.</p>
      </main>
    </div>
  );
}
