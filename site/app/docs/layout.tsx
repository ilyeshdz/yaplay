import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { source } from '@/lib/source';
import { Wordmark } from '@/components/wordmark';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <DocsLayout tree={source.pageTree} nav={{ title: <Wordmark />, url: '/' }}
      themeSwitch={{ enabled: false }} searchToggle={{ enabled: false }}
      links={[{ text: 'Download', url: '/#download' }, { text: 'Release notes', url: '/releases' }, { text: 'GitHub', url: 'https://github.com/ilyeshdz/yaplay', external: true }]}>
      {children}
    </DocsLayout>
  );
}
