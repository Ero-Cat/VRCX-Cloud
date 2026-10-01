import type { ScreenshotExportProgress } from '@/platform/tauri/bindings';

export const SCREENSHOT_EXPORT_SPINNER_DELAY_MS = 600;

export type ScreenshotExportToastView =
    | { kind: 'progress'; percent: number; writtenFiles: number }
    | { kind: 'spinner'; writtenFiles: number };

function clampPercent(value: number) {
    if (!Number.isFinite(value)) {
        return 0;
    }
    return Math.min(100, Math.max(0, Math.round(value)));
}

export function resolveScreenshotExportToastView(
    progress: ScreenshotExportProgress,
    finalizingElapsedMs: number
): ScreenshotExportToastView {
    const percent =
        progress.totalBytes > 0
            ? clampPercent((progress.writtenBytes / progress.totalBytes) * 100)
            : progress.totalFiles > 0
              ? clampPercent(
                    (progress.writtenFiles / progress.totalFiles) * 100
                )
              : 0;

    if (
        progress.finalizing &&
        finalizingElapsedMs >= SCREENSHOT_EXPORT_SPINNER_DELAY_MS
    ) {
        return { kind: 'spinner', writtenFiles: progress.writtenFiles };
    }

    return { kind: 'progress', percent, writtenFiles: progress.writtenFiles };
}
