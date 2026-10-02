import { webInvoke } from './webTransport';

/**
 * Web command transport: every backend call goes to POST /api/invoke.
 */
export async function invokeTauri<TReturn = unknown>(
    command: string,
    args?: Record<string, unknown>
): Promise<TReturn> {
    return webInvoke<TReturn>(command, args);
}
