/**
 * Small JSON file KV the desktop stores in AppCache via the Tauri fs
 * plugin. In the browser the same content lives in localStorage.
 */
import { isWebPlatform } from './webTransport';

export async function readLocalTextFile(fileName: string): Promise<string> {
    if (isWebPlatform()) {
        const value = window.localStorage.getItem(`vrcx-file:${fileName}`);
        if (value === null) {
            throw new Error(`file not found: ${fileName}`);
        }
        return value;
    }
    const fs = await import('@tauri-apps/plugin-fs');
    return fs.readTextFile(fileName, { baseDir: fs.BaseDirectory.AppCache });
}

export async function writeLocalTextFile(
    fileName: string,
    contents: string
): Promise<void> {
    if (isWebPlatform()) {
        window.localStorage.setItem(`vrcx-file:${fileName}`, contents);
        return;
    }
    const fs = await import('@tauri-apps/plugin-fs');
    await fs.mkdir('', {
        baseDir: fs.BaseDirectory.AppCache,
        recursive: true
    });
    await fs.writeTextFile(fileName, contents, {
        baseDir: fs.BaseDirectory.AppCache
    });
}
