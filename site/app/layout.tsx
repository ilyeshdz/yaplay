import type { Metadata } from 'next';
import localFont from 'next/font/local';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';
import { branding, brandAsset } from '@/lib/branding';

const geistPixel = localFont({ src: '../fonts/GeistPixel-Square.ttf', variable: '--font-geist-pixel', display: 'swap', weight: '400' });

export const metadata: Metadata = {
  title: { default: `${branding.name} — ${branding.tagline}`, template: `%s — ${branding.name}` },
  description: branding.description,
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL || 'https://ilyeshdz.github.io'),
  icons: {
    icon: [{ url: brandAsset('icon.svg'), type: 'image/svg+xml' }, { url: brandAsset('icon.ico'), sizes: 'any' }],
    apple: [{ url: brandAsset('apple-touch-icon.png'), sizes: '180x180' }],
  },
  openGraph: {
    title: branding.name,
    description: branding.description,
    type: 'website',
    images: [{ url: brandAsset('banner.png'), width: 1740, height: 702, alt: 'Yaplay' }],
  },
  twitter: { card: 'summary_large_image', title: branding.name, description: branding.description, images: [brandAsset('banner.png')] },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`dark ${geistPixel.variable}`} suppressHydrationWarning>
      <body>
        <RootProvider search={{ enabled: false }} theme={{ forcedTheme: 'dark' }}>
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
