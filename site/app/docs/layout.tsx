import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { source } from '@/lib/source';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <DocsLayout tree={source.pageTree} nav={{ title: 'Yaplay', url: '/' }}
      themeSwitch={{ enabled: false }} searchToggle={{ enabled: false }}
      links={[{ text: 'Download', url: '/#download' }, { text: 'GitHub', url: 'https://github.com/ilyeshdz/yaplay', external: true }]}>
      {children}
    </DocsLayout>
  );
}
