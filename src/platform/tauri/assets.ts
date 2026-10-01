import { convertFileSrc as tauriConvertFileSrc } from '@tauri-apps/api/core';

import { isWebPlatform } from './webTransport';

/**
 * Desktop builds serve local cache files through custom protocols
 * (`vrcx-0-img`, `vrcx-0-thumb`, ...). In the browser the same files are
 * addressed by the server's image route.
 */
export function convertFileSrc(filePath: string, protocol = 'asset'): string {
    if (isWebPlatform()) {
        const match = /([^/]+)\/([0-9]+)\.png$/.exec(filePath);
        if (match) {
            return `/api/img/${match[1]}/${match[2]}`;
        }
        return filePath;
    }
    return tauriConvertFileSrc(filePath, protocol);
}
