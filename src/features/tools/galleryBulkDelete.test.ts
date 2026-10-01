import { describe, expect, it, vi } from 'vitest';

import { runGalleryBulkDelete } from './galleryBulkDelete';

function buildRunner(overrides: {
    assetIds: string[];
    deleteAsset(assetId: string): Promise<unknown>;
    isCancelled?(): boolean;
    isScopeValid?(): boolean;
}) {
    const deletedIds: string[] = [];
    const progressSteps: number[] = [];
    return {
        deletedIds,
        progressSteps,
        run: () =>
            runGalleryBulkDelete({
                isCancelled: () => false,
                isScopeValid: () => true,
                ...overrides,
                onDeleted: (assetId) => deletedIds.push(assetId),
                onProgress: (done) => progressSteps.push(done)
            })
    };
}

describe('runGalleryBulkDelete', () => {
    it('deletes every asset in order and reports progress', async () => {
        const runner = buildRunner({
            assetIds: ['a', 'b', 'c'],
            deleteAsset: () => Promise.resolve()
        });

        const outcome = await runner.run();

        expect(outcome).toEqual({
            cancelled: false,
            deleted: 3,
            failed: 0,
            lastError: ''
        });
        expect(runner.deletedIds).toEqual(['a', 'b', 'c']);
        expect(runner.progressSteps).toEqual([1, 2, 3]);
    });

    it('keeps deleting after a failure and reports the last error', async () => {
        const runner = buildRunner({
            assetIds: ['a', 'b', 'c'],
            deleteAsset: (assetId) =>
                assetId === 'b'
                    ? Promise.reject(new Error('HTTP 500'))
                    : Promise.resolve()
        });

        const outcome = await runner.run();

        expect(outcome.deleted).toBe(2);
        expect(outcome.failed).toBe(1);
        expect(outcome.lastError).toBe('HTTP 500');
        expect(runner.deletedIds).toEqual(['a', 'c']);
    });

    it('stops at the next asset once the run is cancelled', async () => {
        let cancelled = false;
        const runner = buildRunner({
            assetIds: ['a', 'b', 'c'],
            deleteAsset: (assetId) => {
                if (assetId === 'a') {
                    cancelled = true;
                }
                return Promise.resolve();
            },
            isCancelled: () => cancelled
        });

        const outcome = await runner.run();

        expect(outcome.cancelled).toBe(true);
        expect(outcome.deleted).toBe(1);
        expect(runner.deletedIds).toEqual(['a']);
    });

    it('stops without removing assets once the auth scope changes', async () => {
        let scopeValid = true;
        const deleteAsset = vi.fn(() => {
            scopeValid = false;
            return Promise.resolve();
        });
        const runner = buildRunner({
            assetIds: ['a', 'b', 'c'],
            deleteAsset,
            isScopeValid: () => scopeValid
        });

        const outcome = await runner.run();

        expect(deleteAsset).toHaveBeenCalledTimes(1);
        expect(outcome.deleted).toBe(1);
        expect(outcome.cancelled).toBe(false);
        expect(runner.deletedIds).toEqual([]);
    });
});
