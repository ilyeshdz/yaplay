import { brandAsset } from '@/lib/branding';

export function Wordmark() {
  return <img className="wordmark" src={brandAsset('wordmark.svg')} width={172} height={40} alt="Yaplay" />;
}
