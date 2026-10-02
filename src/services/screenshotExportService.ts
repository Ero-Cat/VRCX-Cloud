import type { ScreenshotExportProgress } from '@/platform/native/bindings';

type ScreenshotExportProgressListener = (
    progress: ScreenshotExportProgress
) => void;

const listeners = new Set<ScreenshotExportProgressListener>();

export function handleScreenshotExportProgressEvent(
    progress: ScreenshotExportProgress
): void {
    for (const listener of listeners) {
        listener(progress);
    }
}

export function subscribeScreenshotExportProgress(
    listener: ScreenshotExportProgressListener
): () => void {
    listeners.add(listener);
    return () => {
        listeners.delete(listener);
    };
}
