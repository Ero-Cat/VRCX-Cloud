import { describe, expect, it } from 'vitest';

import {
    buildMyAvatarsGridRows,
    getMyAvatarsGridMetrics,
    getVisibleMyAvatarsGridRows
} from './myAvatarsGrid';

describe('myAvatarsGrid', () => {
    it('lays out more avatar cards per row when the grid has more horizontal space', () => {
        const narrow = getMyAvatarsGridMetrics({
            gridDensity: 'standard',
            width: 320
        });
        const wide = getMyAvatarsGridMetrics({
            gridDensity: 'standard',
            width: 980
        });

        expect(narrow.gridColumnCount).toBe(1);
        expect(wide.gridColumnCount).toBeGreaterThan(narrow.gridColumnCount);
        expect(wide.gridGap).toBe(4);
        expect(wide.gridMinWidth).toBe(184);
    });

    it('fits more columns when the user selects a denser grid', () => {
        const standard = getMyAvatarsGridMetrics({
            gridDensity: 'standard',
            width: 640
        });
        const dense = getMyAvatarsGridMetrics({
            gridDensity: 'dense',
            width: 640
        });

        expect(dense.gridColumnCount).toBeGreaterThan(standard.gridColumnCount);
        expect(dense.gridMinWidth).toBe(129);
    });

    it('sizes grid cells from the shared 4:3 avatar image ratio', () => {
        const metrics = getMyAvatarsGridMetrics({
            gridDensity: 'standard',
            width: 800
        });
        const columnWidth =
            (800 - metrics.gridGap * (metrics.gridColumnCount - 1)) /
            metrics.gridColumnCount;
        const cardWidth = columnWidth - metrics.gridPadding * 2;

        expect(metrics.cellHeight).toBe(
            Math.round(cardWidth / (4 / 3)) + metrics.gridPadding * 2
        );
    });

    it('groups avatars into stable virtual rows and drops the trailing gap', () => {
        const avatars = [
            { id: 'avtr_1' },
            { id: 'avtr_2' },
            { id: 'avtr_3' },
            { id: 'avtr_4' },
            { id: 'avtr_5' }
        ];

        expect(
            buildMyAvatarsGridRows({
                avatars,
                cellHeight: 240,
                gridColumnCount: 2,
                gridGap: 8
            }).rows
        ).toEqual([
            {
                key: 'grid-row:0',
                avatars: [{ id: 'avtr_1' }, { id: 'avtr_2' }],
                cellHeight: 240,
                top: 0,
                height: 248
            },
            {
                key: 'grid-row:2',
                avatars: [{ id: 'avtr_3' }, { id: 'avtr_4' }],
                cellHeight: 240,
                top: 248,
                height: 248
            },
            {
                key: 'grid-row:4',
                avatars: [{ id: 'avtr_5' }],
                cellHeight: 240,
                top: 496,
                height: 240
            }
        ]);
    });

    it('keeps nearby virtual rows mounted around the visible scroll window', () => {
        const gridRows = buildMyAvatarsGridRows({
            avatars: Array.from({ length: 20 }, (_, index) => ({
                id: `avtr_${index}`
            })),
            cellHeight: 200,
            gridColumnCount: 2,
            gridGap: 0
        }).rows;

        const visibleRows = getVisibleMyAvatarsGridRows({
            gridRows,
            scrollTop: 800,
            viewportHeight: 400
        });

        expect(visibleRows[0]?.top).toBeLessThanOrEqual(400);
        expect(visibleRows.at(-1)?.top).toBeGreaterThanOrEqual(1400);
        expect(visibleRows.length).toBeLessThan(gridRows.length);
    });

    it('returns no visible rows while grid rows are not ready yet', () => {
        expect(
            getVisibleMyAvatarsGridRows({
                gridRows: null,
                scrollTop: 0,
                viewportHeight: 400
            })
        ).toEqual([]);
    });
});
