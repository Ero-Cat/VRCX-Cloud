import { languageCodes } from '@/localization/locales';
import { commands } from '@/platform/tauri/bindings';
import { isRecord } from '@/shared/utils/record';

import i18n from './i18nService';

const WATCH_INTERVAL_MS = 1200;

let watchTimer: number | null = null;
let watchGeneration = 0;
let lastContent = '';

export type I18nWatchResult = { error: string | null; loadedAt?: string };

export function detectLangFromPath(filePath: string): string | null {
    const base = filePath
        .replace(/\\/g, '/')
        .split('/')
        .pop()
        ?.replace(/\.json$/i, '');
    if (!base) return null;
    return (
        languageCodes.find((c) => c.toLowerCase() === base.toLowerCase()) ??
        null
    );
}

export function isI18nWatchActive(): boolean {
    return watchTimer !== null;
}

export async function loadI18nFromFile(
    filePath: string,
    targetLang: string
): Promise<void> {
    const content = await commands.appDevkitReadFile(filePath);
    if (content === lastContent) return;
    lastContent = content;
    const data: unknown = JSON.parse(content);
    if (!isRecord(data)) {
        throw new TypeError('Translation file must contain a JSON object');
    }
    i18n.addResourceBundle(targetLang, 'translation', data, true, true);
    await i18n.changeLanguage(targetLang);
}

export function startI18nWatch(
    filePath: string,
    targetLang: string,
    onResult: (result: I18nWatchResult) => void
): void {
    stopI18nWatch();
    watchGeneration += 1;
    lastContent = '';
    const generation = watchGeneration;

    const tryLoad = () => {
        if (generation !== watchGeneration) return;
        void loadI18nFromFile(filePath, targetLang)
            .then(() =>
                onResult({
                    error: null,
                    loadedAt: new Date().toLocaleTimeString()
                })
            )
            .catch((err) =>
                onResult({
                    error:
                        err instanceof Error
                            ? err.message
                            : 'Failed to load file'
                })
            );
    };

    tryLoad();
    watchTimer = window.setInterval(tryLoad, WATCH_INTERVAL_MS);
}

export function stopI18nWatch(): void {
    watchGeneration += 1;
    lastContent = '';
    if (watchTimer !== null) {
        window.clearInterval(watchTimer);
        watchTimer = null;
    }
}
