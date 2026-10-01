import { describe, expect, it } from 'vitest';

import type { ScreenshotExportProgress } from '@/platform/tauri/bindings';

import {
    resolveScreenshotExportToastView,
    SCREENSHOT_EXPORT_SPINNER_DELAY_MS
} from './screenshotExportToastView';

function progress(
    overrides: Partial<ScreenshotExportProgress> = {}
): ScreenshotExportProgress {
    return {
        running: true,
        finalizing: false,
        totalFiles: 4,
        writtenFiles: 0,
        skippedFiles: 0,
        totalBytes: 1000,
        writtenBytes: 0,
        cancelled: false,
        error: null,
        outputPath: null,
        ...overrides
    };
}

describe('resolveScreenshotExportToastView', () => {
    it('shows a byte-based percentage while files are being written', () => {
        expect(
            resolveScreenshotExportToastView(
                progress({ writtenBytes: 250, writtenFiles: 1 }),
                0
            )
        ).toEqual({ kind: 'progress', percent: 25, writtenFiles: 1 });

        expect(
            resolveScreenshotExportToastView(
                progress({ writtenBytes: 1000, writtenFiles: 4 }),
                0
            )
        ).toEqual({ kind: 'progress', percent: 100, writtenFiles: 4 });
    });

    it('falls back to a file count percentage when byte totals are unknown', () => {
        expect(
            resolveScreenshotExportToastView(
                progress({ totalBytes: 0, writtenFiles: 1 }),
                0
            )
        ).toEqual({ kind: 'progress', percent: 25, writtenFiles: 1 });
    });

    it('keeps the full bar during a short finalize and only then spins', () => {
        const finalizing = progress({
            finalizing: true,
            writtenBytes: 1000,
            writtenFiles: 4
        });

        expect(resolveScreenshotExportToastView(finalizing, 0)).toEqual({
            kind: 'progress',
            percent: 100,
            writtenFiles: 4
        });
        expect(
            resolveScreenshotExportToastView(
                finalizing,
                SCREENSHOT_EXPORT_SPINNER_DELAY_MS - 1
            )
        ).toEqual({ kind: 'progress', percent: 100, writtenFiles: 4 });
        expect(
            resolveScreenshotExportToastView(
                finalizing,
                SCREENSHOT_EXPORT_SPINNER_DELAY_MS
            )
        ).toEqual({ kind: 'spinner', writtenFiles: 4 });
    });

    it('never reports a percentage outside 0-100', () => {
        expect(
            resolveScreenshotExportToastView(
                progress({ writtenBytes: 5000, totalBytes: 1000 }),
                0
            )
        ).toEqual({ kind: 'progress', percent: 100, writtenFiles: 0 });
        expect(
            resolveScreenshotExportToastView(
                progress({ totalBytes: 0, totalFiles: 0 }),
                0
            )
        ).toEqual({ kind: 'progress', percent: 0, writtenFiles: 0 });
    });
});
