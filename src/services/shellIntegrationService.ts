import { commands } from '@/platform/native/bindings';
import type { AppDataDirState } from '@/platform/native/bindings';

export async function openExternalLink(url: string): Promise<void> {
    window.open(url, '_blank', 'noopener,noreferrer');
}

export async function restartApplication(): Promise<void> {
    await commands.appRestartApplication();
}

export async function getAppDataDirState(): Promise<AppDataDirState> {
    return commands.appGetAppDataDirState();
}

export async function getClipboardText(): Promise<string> {
    const value = await commands.appGetClipboard().catch(() => '');
    return typeof value === 'string' ? value : '';
}

export async function setTrayIconNotification(notify: boolean): Promise<void> {
    await commands.appSetTrayIconNotification(notify);
}

export async function setTaskbarOverlayNotification(
    notify: boolean
): Promise<void> {
    await commands.appSetTaskbarOverlayNotification(notify);
}

/**
 * Web build: folder/file locations live on the server host. Ask for the
 * path directly instead of an OS dialog.
 */
export async function openFolderSelectorDialog(
    defaultPath: string
): Promise<string> {
    const input = window.prompt('Folder path on the server', defaultPath || '');
    return input === null ? '' : input.trim();
}

export async function openFileSelectorDialog(
    defaultPath: string,
    defaultExt: string,
    _defaultFilter: string
): Promise<string> {
    void _defaultFilter;
    const input = window.prompt(
        `File path on the server (*${defaultExt})`,
        defaultPath || ''
    );
    return input === null ? '' : input.trim();
}

export async function saveFileSelectorDialog(
    defaultPath: string,
    defaultName: string,
    defaultExt: string,
    defaultFilter: string
): Promise<string> {
    const selected = await commands.appSaveFileSelectorDialog(
        defaultPath,
        defaultName,
        defaultExt,
        defaultFilter
    );
    return typeof selected === 'string' ? selected : '';
}

export async function openCalendarFile(icsContent: string): Promise<void> {
    await commands.appOpenCalendarFile(icsContent);
}

export async function saveCalendarFile(
    defaultName: string,
    icsContent: string
): Promise<void> {
    await commands.appSaveCalendarFile(defaultName, icsContent);
}

export async function saveJsonFile(
    defaultName: string,
    json: string
): Promise<void> {
    await commands.appSaveVrcRegJsonFile(null, defaultName, json);
}

export async function readVrchatConfigFileSafe(): Promise<string> {
    const config = await commands.appReadConfigFileSafe();
    return typeof config === 'string' ? config : '';
}

export async function writeVrchatConfigFile(json: string): Promise<void> {
    await commands.appWriteConfigFile(json);
}

export async function vrchatCacheLocationWouldChange(
    json: string
): Promise<boolean> {
    return commands.appVrchatCacheLocationWouldChange(json);
}

export async function writeVrchatConfigFileWithCacheCleanup(
    json: string
): Promise<string | null> {
    const result = await commands.appWriteConfigFileWithCacheCleanup(json);
    return result.oldCacheCleanupError;
}

export async function getVrchatUserModeration(
    currentUserId: string,
    userId: string
): Promise<number> {
    return commands.appGetVrchatUserModeration(currentUserId, userId);
}

export async function setVrchatUserModeration(
    currentUserId: string,
    userId: string,
    moderationType: string | number
): Promise<boolean> {
    return commands.appSetVrchatUserModeration(
        currentUserId,
        userId,
        Number(moderationType)
    );
}

export async function openDiscordProfile(discordId: string): Promise<void> {
    await commands.appOpenDiscordProfile(discordId);
}
