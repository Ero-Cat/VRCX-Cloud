import { toast } from '@/services/toastService';
import { Progress } from '@/ui/shadcn/progress';

const GALLERY_BULK_PROGRESS_TOAST_ID = 'gallery-bulk-progress';

export type GalleryBulkProgressToast = {
    update(done: number): void;
    dismiss(): void;
};

export function startGalleryBulkProgressToast({
    total,
    buildMessage,
    cancelLabel,
    onCancel
}: {
    total: number;
    buildMessage(done: number): string;
    cancelLabel: string;
    onCancel(): void;
}): GalleryBulkProgressToast {
    let cancelled = false;

    function render(done: number) {
        if (cancelled) {
            return;
        }
        toast.add({
            type: 'loading',
            title: buildMessage(done),
            id: GALLERY_BULK_PROGRESS_TOAST_ID,
            timeout: 0,
            description: (
                <Progress
                    value={total > 0 ? Math.round((done / total) * 100) : 0}
                />
            ),
            actionProps: {
                children: cancelLabel,
                onClick: () => {
                    cancelled = true;
                    onCancel();
                }
            }
        });
    }

    render(0);

    return {
        update: render,
        dismiss() {
            cancelled = true;
            toast.close(GALLERY_BULK_PROGRESS_TOAST_ID);
        }
    };
}
