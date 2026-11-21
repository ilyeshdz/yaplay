'use client';

import { useEffect, useState } from 'react';
import { Download } from 'lucide-react';
import { detectOS, downloads, downloadUrl } from '@/lib/downloads';

export function DownloadButton() {
  const [os, setOS] = useState<ReturnType<typeof detectOS>>('unknown');
  useEffect(() => { setOS(detectOS(navigator.userAgent, navigator.platform, navigator.maxTouchPoints)); }, []);
  const build = downloads.find((item) => item.id === (os === 'windows' ? 'windows' : 'mac-arm'))!;
  return (
    <div className="detected-download">
      {os === 'mac' || os === 'windows' ? (
        <a className="home-action primary-link" href={downloadUrl(build.asset)}><Download size={16} aria-hidden="true" />Download for {build.label}</a>
      ) : os === 'linux' || os === 'mobile' ? (
        <span className="home-action unsupported" role="status">{os === 'linux' ? 'Linux is not supported yet' : 'Available for macOS and Windows'}</span>
      ) : (
        <a className="home-action primary-link" href="#download"><Download size={16} aria-hidden="true" />Choose your download</a>
      )}
      {os === 'mac' && <p className="download-hint" id="mac-download-hint">Apple Silicon only · macOS</p>}
    </div>
  );
}
