export type GalleryBulkDeleteOutcome = {
    cancelled: boolean;
    deleted: number;
    failed: number;
    lastError: string;
};

export async function runGalleryBulkDelete({
    assetIds,
    deleteAsset,
    isCancelled,
    isScopeValid,
    onDeleted,
    onProgress
}: {
    assetIds: string[];
    deleteAsset(assetId: string): Promise<unknown>;
    isCancelled(): boolean;
    isScopeValid(): boolean;
    onDeleted(assetId: string): void;
    onProgress(done: number): void;
}): Promise<GalleryBulkDeleteOutcome> {
    let deleted = 0;
    let failed = 0;
    let lastError = '';

    for (const assetId of assetIds) {
        if (isCancelled() || !isScopeValid()) {
            break;
        }
        try {
            await deleteAsset(assetId);
            deleted += 1;
            if (isScopeValid()) {
                onDeleted(assetId);
            }
        } catch (error) {
            failed += 1;
            lastError = error instanceof Error ? error.message : lastError;
        }
        onProgress(deleted + failed);
    }

    return {
        cancelled: isCancelled(),
        deleted,
        failed,
        lastError
    };
}
