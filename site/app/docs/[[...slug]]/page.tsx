import { source } from '@/lib/source';
import { DocsPage, DocsBody, DocsTitle, DocsDescription } from 'fumadocs-ui/page';
import defaultComponents from 'fumadocs-ui/mdx';
import { notFound } from 'next/navigation';

export const dynamicParams = false;
export function generateStaticParams() { return source.generateParams(); }

export async function generateMetadata(props: { params: Promise<{ slug?: string[] }> }) {
  const { slug } = await props.params;
  const page = source.getPage(slug);
  if (!page) notFound();
  return { title: page.data.title, description: page.data.description };
}

export default async function Page(props: { params: Promise<{ slug?: string[] }> }) {
  const { slug } = await props.params;
  const page = source.getPage(slug);
  if (!page) notFound();
  const Content = page.data.body;
  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <DocsTitle>{page.data.title}</DocsTitle>
      {page.data.description && <DocsDescription>{page.data.description}</DocsDescription>}
      <DocsBody><Content components={{ ...defaultComponents, WrapperDownload: () => (
        <a href={`${process.env.NEXT_PUBLIC_BASE_PATH || ''}/yaplay-client.mjs`} download>downloadable JavaScript wrapper</a>
      ) }} /></DocsBody>
    </DocsPage>
  );
}
