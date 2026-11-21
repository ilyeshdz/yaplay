import Link from 'next/link';
import { OverlayPreview } from '@/components/overlay-preview';

const releases = 'https://github.com/ilyeshdz/yaplay/releases';
export default function Home() {
  return (
    <div className="home">
      <header className="home-nav">
        <Link className="wordmark" href="/">Yaplay</Link>
        <nav aria-label="Main navigation"><a href="#download">Download</a><Link href="/docs">Docs</Link><a href="https://github.com/ilyeshdz/yaplay">GitHub</a></nav>
      </header>
      <main id="main">
        <section className="intro">
          <h1>A desktop overlay for your app.</h1>
          <p>See who’s talking and receive notifications above your windows. Connect any application with a plugin, a wrapper, or a local WebSocket.</p>
          <div className="home-actions"><a className="primary-link" href="#download">Download Yaplay</a><Link href="/docs">Integration guide</Link></div>
        </section>
        <OverlayPreview />
        <section className="downloads" id="download">
          <h2>Download</h2>
          <p>Choose your build in GitHub Releases, open Yaplay, then press Start.</p>
          <div className="download-options">
            <a href={releases}>Windows <span>x86_64</span></a>
            <a href={releases}>macOS <span>Apple Silicon</span></a>
            <a href={releases}>macOS <span>Intel</span></a>
          </div>
          <p className="small">macOS builds are not notarized. If a build isn’t available yet, <Link href="/docs/installation">build from source</Link>.</p>
        </section>
      </main>
      <footer>Independent of any app or ecosystem. <Link href="/docs/integration">Build an integration.</Link></footer>
    </div>
  );
}
