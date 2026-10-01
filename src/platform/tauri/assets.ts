import { convertFileSrc as tauriConvertFileSrc } from '@tauri-apps/api/core';

export function convertFileSrc(filePath: string, protocol = 'asset'): string {
    return tauriConvertFileSrc(filePath, protocol);
}
