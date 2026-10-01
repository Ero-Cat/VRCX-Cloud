import { isWebPlatform, webInvoke } from './webTransport';

export async function invokeTauri<TReturn = unknown>(
    command: string,
    args?: Record<string, unknown>
): Promise<TReturn> {
    if (isWebPlatform()) {
        return webInvoke<TReturn>(command, args);
    }
    const core = await import('@tauri-apps/api/core');
    return core.invoke<TReturn>(command, args);
}
