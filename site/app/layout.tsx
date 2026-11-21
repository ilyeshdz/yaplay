import type { Metadata } from 'next';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';

export const metadata: Metadata = {
  title: { default: 'Yaplay', template: '%s — Yaplay' },
  description: 'An independent desktop overlay for voice activity and notifications. Connect any application over a local WebSocket.',
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
