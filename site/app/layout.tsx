import type { Metadata } from 'next';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';
import { branding } from '@/lib/branding';

export const metadata: Metadata = {
  title: { default: `${branding.name} — ${branding.tagline}`, template: `%s — ${branding.name}` },
  description: branding.description,
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body>
        <RootProvider search={{ enabled: false }} theme={{ forcedTheme: 'dark' }}>
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
