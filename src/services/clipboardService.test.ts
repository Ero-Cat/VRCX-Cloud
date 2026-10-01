import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AppToastOptions } from '@/services/toastService';

const toastMocks = vi.hoisted(() => ({
    error: vi.fn(),
    success: vi.fn()
}));

vi.mock('@/services/toastService', () => ({
    toast: {
        add: (options: AppToastOptions) => {
            switch (options.type) {
                case 'error':
                    return toastMocks.error(options);
                case 'success':
                    return toastMocks.success(options);
                default:
                    throw new Error('Unhandled toast type: ' + options.type);
            }
        }
    }
}));

import { copyTextToClipboard } from './clipboardService';

describe('copyTextToClipboard', () => {
    let writeText: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        writeText = vi.fn().mockResolvedValue(undefined);
        vi.stubGlobal('navigator', {
            clipboard: {
                writeText
            }
        });
        toastMocks.error.mockClear();
        toastMocks.success.mockClear();
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('copies text and shows the success toast after the write succeeds', async () => {
        const copied = await copyTextToClipboard('wrld_123', {
            successMessage: 'World ID copied'
        });

        expect(copied).toBe(true);
        expect(writeText).toHaveBeenCalledWith('wrld_123');
        expect(toastMocks.success).toHaveBeenCalledWith(
            expect.objectContaining({
                type: 'success',
                title: 'World ID copied'
            })
        );
        expect(toastMocks.error).not.toHaveBeenCalled();
    });

    it('copies text without showing a success toast when no message is provided', async () => {
        const copied = await copyTextToClipboard('usr_123');

        expect(copied).toBe(true);
        expect(writeText).toHaveBeenCalledWith('usr_123');
        expect(toastMocks.success).not.toHaveBeenCalled();
        expect(toastMocks.error).not.toHaveBeenCalled();
    });

    it('returns false without showing success when the clipboard write fails', async () => {
        writeText.mockRejectedValueOnce(new Error('permission denied'));

        const copied = await copyTextToClipboard('grp_123');

        expect(copied).toBe(false);
        expect(toastMocks.success).not.toHaveBeenCalled();
        expect(toastMocks.error).not.toHaveBeenCalled();
    });

    it('shows a string error toast when the clipboard write fails', async () => {
        writeText.mockRejectedValueOnce(new Error('permission denied'));

        const copied = await copyTextToClipboard('avtr_123', {
            errorMessage: 'Copy failed'
        });

        expect(copied).toBe(false);
        expect(toastMocks.success).not.toHaveBeenCalled();
        expect(toastMocks.error).toHaveBeenCalledWith(
            expect.objectContaining({ type: 'error', title: 'Copy failed' })
        );
    });

    it('shows a resolved error toast when the clipboard write fails', async () => {
        writeText.mockRejectedValueOnce(new Error('permission denied'));

        const copied = await copyTextToClipboard('https://vrchat.com', {
            errorMessage: (error) =>
                error instanceof Error ? error.message : 'Copy failed'
        });

        expect(copied).toBe(false);
        expect(toastMocks.success).not.toHaveBeenCalled();
        expect(toastMocks.error).toHaveBeenCalledWith(
            expect.objectContaining({
                type: 'error',
                title: 'permission denied'
            })
        );
    });
});
