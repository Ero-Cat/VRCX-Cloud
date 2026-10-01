import { recordErrorLog } from '../../services/errorLogService';
import { notifySQLiteError } from '../../shared/sqliteErrorEvents';
import { normalizePlatformError } from './errors';
import { invokeTauri } from './invoke';
import { isWebPlatform, WebCommandError } from './webTransport';

export interface CommandPromise<TResult> extends Promise<TResult> {
    catch<TResult2 = never>(
        onrejected?:
            | ((reason: Error) => TResult2 | PromiseLike<TResult2>)
            | null
    ): Promise<TResult | TResult2>;
}

export function invoke<TReturn = unknown>(
    command: string,
    args?: Record<string, unknown>
): CommandPromise<TReturn> {
    return invokeTauri<TReturn>(command, args).catch((error) => {
        if (error instanceof WebCommandError) {
            if (
                error.code === 'unauthorized' &&
                isWebPlatform() &&
                typeof window !== 'undefined'
            ) {
                window.location.reload();
            }
            // Desktop-only commands degrade silently on the web; the UI
            // hides them behind host capabilities.
            if (error.code === 'unsupportedOnWeb') {
                throw error;
            }
        }
        const normalizedError = normalizePlatformError(
            error,
            `Tauri command failed: ${command}`
        );

        recordErrorLog('rust:command', [
            `command: ${command}`,
            normalizedError
        ]);
        notifySQLiteError(normalizedError);

        throw normalizedError;
    });
}
