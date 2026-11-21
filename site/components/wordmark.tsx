import { brandAsset } from '@/lib/branding';

export function Wordmark() {
  return <img className="wordmark" src={brandAsset('wordmark.svg')} width={120} height={28} alt="Yaplay" />;
}
