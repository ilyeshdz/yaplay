export const branding = {
  name: 'Yaplay',
  tagline: 'An unofficial overlay for Yapin.',
  description: 'An unofficial overlay for Yapin. See call activity and notifications through client modifications such as Yapmod and Yap++.',
};

export function brandAsset(file: string) {
  return `${process.env.NEXT_PUBLIC_BASE_PATH || ''}/branding/${file}`;
}
