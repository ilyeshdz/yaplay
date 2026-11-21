import Link from 'next/link';
import { OverlayPreview } from '@/components/overlay-preview';
import { Wordmark } from '@/components/wordmark';
import { branding, brandAsset } from '@/lib/branding';

const releases = 'https://github.com/ilyeshdz/yaplay/releases';
export default function Home() {
  return (
    <div className="home">
      <header className="home-nav">
        <Link href="/" aria-label="Yaplay home"><Wordmark /></Link>
        <nav aria-label="Main navigation"><a href="#download">Download</a><Link href="/docs">Docs</Link><a href="https://github.com/ilyeshdz/yaplay">GitHub</a></nav>
      </header>
      <main id="main">
        <div className="brand-banner"><img src={brandAsset('banner.svg')} width={870} height={351} alt="Yaplay — an unofficial overlay for Yapin" fetchPriority="high" /></div>
        <section className="intro">
          <h1>{branding.tagline}</h1>
          <p>See who’s talking on Yapin and receive notifications above your windows. Connect through a compatible integration in a Yapin client modification, such as Yapmod or Yap++.</p>
          <div className="home-actions"><a className="primary-link" href="#download">Download Yaplay</a><Link href="/docs">Integration guide</Link></div>
        </section>
        <OverlayPreview />
        <section className="downloads" id="download">
          <h2>Download</h2>
          <p>Choose your build in GitHub Releases, open Yaplay, then press Start. Enable the Yaplay integration in your modified Yapin client to display your call.</p>
          <div className="download-options">
            <a href={releases}>Windows <span>x86_64</span></a>
            <a href={releases}>macOS <span>Apple Silicon</span></a>
            <a href={releases}>macOS <span>Intel</span></a>
          </div>
          <p className="small">macOS builds are not notarized. If a build isn’t available yet, <Link href="/docs/installation">build from source</Link>.</p>
        </section>
      </main>
      <footer>{branding.tagline} <Link href="/docs/integration">Build a client integration.</Link></footer>
    </div>
  );
}
