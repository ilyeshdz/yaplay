import Link from 'next/link';
import { ArrowRight, ArrowUpRight, Bell, Download, Layers, Mic } from 'lucide-react';
import { OverlayPreview } from '@/components/overlay-preview';
import { Wordmark } from '@/components/wordmark';

const releases = 'https://github.com/ilyeshdz/yaplay/releases';
const features = [
  { title: 'Your call, in view.', text: 'See who’s talking, who’s muted, and who’s in your Yapin call above your other windows.', icon: Mic, href: '/docs', action: 'Explore Yaplay' },
  { title: 'Keep up.', text: 'Catch call activity and notifications in a small overlay while you stay with what you’re doing.', icon: Bell, href: '/docs/installation', action: 'Get started' },
  { title: 'Make the connection.', text: 'Connect through a compatible integration in Yapmod, Yap++, or another modified Yapin client.', icon: Layers, href: '/docs/integration', action: 'Build an integration' },
];

export default function Home() {
  return (
    <div className="home">
      <header className="home-nav home-wrap">
        <Link href="/" aria-label="Yaplay home"><Wordmark /></Link>
        <nav aria-label="Main navigation"><a href="#download">Download</a><Link href="/docs">Docs</Link><a href="https://github.com/ilyeshdz/yaplay">GitHub</a></nav>
      </header>
      <main id="main">
        <section className="pixel-hero">
          <div className="intro home-wrap">
            <p className="eyebrow">A little overlay. A closer connection.</p>
            <h1>Your call.<br /><span>Always in view.</span></h1>
            <p className="intro-description">Keep your Yapin call and notifications above your windows. A simple, unofficial overlay for macOS and Windows.</p>
            <div className="home-actions">
              <a className="home-action primary-link" href="#download"><Download size={16} aria-hidden="true" />Download Yaplay</a>
              <Link className="home-action secondary-link" href="/docs">Get started<ArrowRight size={16} aria-hidden="true" /></Link>
            </div>
            <div className="hero-links"><a className="home-link" href={releases}>Release notes<ArrowRight size={14} aria-hidden="true" /></a><Link className="home-link" href="/docs/integration">Build an integration<ArrowRight size={14} aria-hidden="true" /></Link></div>
          </div>
        </section>
        <section className="features home-wrap" aria-labelledby="features-heading">
          <h2 id="features-heading">A place for your call.</h2>
          <div className="feature-grid">
            {features.map(({ title, text, icon: Icon, href, action }) => (
              <div className="feature" key={title}>
                <Icon className="feature-icon" size={20} aria-hidden="true" />
                <h3>{title}</h3><p>{text}</p>
                <Link className="home-link" href={href}>{action}<ArrowRight size={14} aria-hidden="true" /></Link>
              </div>
            ))}
          </div>
        </section>
        <section className="preview-section" aria-labelledby="preview-heading">
          <div className="home-wrap">
            <div className="section-heading"><h2 id="preview-heading">Stay in the conversation.</h2><p>Your call in one corner. Notifications in the other.</p></div>
            <div className="desktop-preview"><OverlayPreview /></div>
          </div>
        </section>
        <section className="downloads home-wrap" id="download">
          <h2>A little closer, on your desktop.</h2>
          <p>Choose your build in GitHub Releases, open Yaplay, then press Start. Connect a compatible integration in your modified Yapin client to display your call.</p>
          <div className="download-options">
            <a href={releases}><div>Windows<span>x86_64</span></div><ArrowUpRight size={18} aria-hidden="true" /></a>
            <a href={releases}><div>macOS<span>Apple Silicon</span></div><ArrowUpRight size={18} aria-hidden="true" /></a>
            <a href={releases}><div>macOS<span>Intel</span></div><ArrowUpRight size={18} aria-hidden="true" /></a>
          </div>
          <p className="small">macOS builds are not notarized. If a build isn’t available yet, <Link href="/docs/installation">build from source</Link>.</p>
        </section>
      </main>
      <footer className="home-wrap"><p>An independent overlay for Yapin, built for your desktop.</p><nav aria-label="Community"><a className="home-link" href="https://github.com/ilyeshdz/yaplay">GitHub<ArrowUpRight size={14} aria-hidden="true" /></a><a className="home-link" href="https://github.com/ilyeshdz/yaplay/issues">Issues<ArrowUpRight size={14} aria-hidden="true" /></a></nav></footer>
    </div>
  );
}
