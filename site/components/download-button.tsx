'use client';

import { useEffect, useState } from 'react';
import { Download } from 'lucide-react';
import { detectOS, downloads, downloadUrl } from '@/lib/downloads';

export function DownloadButton() {
  const [os, setOS] = useState<ReturnType<typeof detectOS>>('unknown');
  const [chip, setChip] = useState('mac-arm');
  useEffect(() => { setOS(detectOS(navigator.userAgent, navigator.platform, navigator.maxTouchPoints)); }, []);
  const build = downloads.find((item) => item.id === (os === 'windows' ? 'windows' : chip))!;
  return (
    <div className="detected-download">
      {os === 'mac' && <label className="chip-choice">Mac chip
        <select value={chip} onChange={(event) => setChip(event.target.value)}>
          <option value="mac-arm">Apple Silicon</option><option value="mac-intel">Intel</option>
        </select>
      </label>}
      {os === 'windows' || os === 'mac' ? (
        <a className="home-action primary-link" href={downloadUrl(build.asset)}><Download size={16} aria-hidden="true" />Download for {build.label}</a>
      ) : os === 'linux' || os === 'mobile' ? (
        <span className="home-action unsupported" role="status">{os === 'linux' ? 'Linux is not supported yet' : 'Available for macOS and Windows'}</span>
      ) : (
        <a className="home-action primary-link" href="#download"><Download size={16} aria-hidden="true" />Choose your download</a>
      )}
      {os === 'mac' && <p className="download-hint">Check your chip in Apple menu → About This Mac.</p>}
    </div>
  );
}
