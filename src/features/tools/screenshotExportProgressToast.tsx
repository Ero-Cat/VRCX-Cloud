import type { ScreenshotExportProgress } from '@/platform/tauri/bindings';
import { toast } from '@/services/toastService';
import { Progress } from '@/ui/shadcn/progress';
import { Spinner } from '@/ui/shadcn/spinner';

import {
    resolveScreenshotExportToastView,
    SCREENSHOT_EXPORT_SPINNER_DELAY_MS
} from './screenshotExportToastView';

const SCREENSHOT_EXPORT_TOAST_ID = 'screenshot-export-progress';

export type ScreenshotExportProgressToast = {
    update(progress: ScreenshotExportProgress): void;
    dismiss(): void;
};

export function startScreenshotExportProgressToast({
    buildMessage,
    finalizingLabel,
    cancelLabel,
    onCancel
}: {
    buildMessage(writtenFiles: number, totalFiles: number): string;
    finalizingLabel: string;
    cancelLabel: string;
    onCancel(): void;
}): ScreenshotExportProgressToast {
    let dismissed = false;
    let finalizingStartedAt = 0;
    let spinnerTimer: number | null = null;
    let latestProgress: ScreenshotExportProgress | null = null;

    function clearSpinnerTimer() {
        if (spinnerTimer !== null) {
            window.clearTimeout(spinnerTimer);
            spinnerTimer = null;
        }
    }

    function render() {
        if (dismissed || !latestProgress) {
            return;
        }
        const view = resolveScreenshotExportToastView(
            latestProgress,
            finalizingStartedAt ? Date.now() - finalizingStartedAt : 0
        );
        toast.add({
            type: 'loading',
            title:
                view.kind === 'spinner'
                    ? finalizingLabel
                    : buildMessage(
                          view.writtenFiles,
                          latestProgress.totalFiles
                      ),
            id: SCREENSHOT_EXPORT_TOAST_ID,
            timeout: 0,
            description:
                view.kind === 'spinner' ? (
                    <Spinner />
                ) : (
                    <Progress value={view.percent} />
                ),
            actionProps: { children: cancelLabel, onClick: onCancel }
        });
    }

    return {
        update(progress) {
            latestProgress = progress;
            if (progress.finalizing && finalizingStartedAt === 0) {
                finalizingStartedAt = Date.now();
                clearSpinnerTimer();
                spinnerTimer = window.setTimeout(
                    render,
                    SCREENSHOT_EXPORT_SPINNER_DELAY_MS
                );
            }
            render();
        },
        dismiss() {
            dismissed = true;
            clearSpinnerTimer();
            toast.close(SCREENSHOT_EXPORT_TOAST_ID);
        }
    };
}
