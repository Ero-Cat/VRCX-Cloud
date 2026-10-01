import { useEffect, useMemo } from 'react';

import { useScrollViewportMetrics } from '@/lib/useScrollViewportMetrics';

import {
    buildMyAvatarsGridRows,
    getMyAvatarsGridMetrics,
    getVisibleMyAvatarsGridRows
} from './myAvatarsGrid';
import type {
    MyAvatarRow,
    MyAvatarsGridDensity,
    MyAvatarsPlatformFilter,
    MyAvatarsReleaseStatusFilter,
    MyAvatarsViewMode
} from './myAvatarsTypes';

const MY_AVATARS_GRID_HORIZONTAL_INSET = 8;

export function useMyAvatarsGridVirtualization({
    deferredSearchQuery,
    filteredAvatars,
    gridDensity,
    platformFilter,
    releaseStatusFilter,
    tagFilters,
    viewMode
}: {
    deferredSearchQuery: string;
    filteredAvatars: MyAvatarRow[];
    gridDensity: MyAvatarsGridDensity;
    platformFilter: MyAvatarsPlatformFilter;
    releaseStatusFilter: MyAvatarsReleaseStatusFilter;
    tagFilters: Set<string>;
    viewMode: MyAvatarsViewMode;
}) {
    const {
        resetScrollTop,
        viewportMetrics: gridScrollMetrics,
        viewportRef: gridScrollRef
    } = useScrollViewportMetrics({ enabled: viewMode === 'grid' });

    useEffect(() => {
        if (viewMode !== 'grid') {
            return;
        }

        resetScrollTop();
    }, [
        deferredSearchQuery,
        filteredAvatars.length,
        gridDensity,
        platformFilter,
        resetScrollTop,
        releaseStatusFilter,
        tagFilters,
        viewMode
    ]);

    const {
        densityConfig,
        gridGap,
        gridMinWidth,
        gridPadding,
        gridColumnCount,
        cellHeight
    } = getMyAvatarsGridMetrics({
        gridDensity,
        width: Math.max(
            0,
            gridScrollMetrics.width - MY_AVATARS_GRID_HORIZONTAL_INSET
        )
    });
    const positionedRows = useMemo(
        () =>
            buildMyAvatarsGridRows({
                avatars: filteredAvatars,
                cellHeight,
                gridColumnCount,
                gridGap
            }),
        [cellHeight, filteredAvatars, gridColumnCount, gridGap]
    );
    const visibleGridRows = useMemo(
        () =>
            getVisibleMyAvatarsGridRows({
                gridRows: positionedRows.rows,
                scrollTop: gridScrollMetrics.scrollTop,
                viewportHeight: gridScrollMetrics.viewportHeight
            }),
        [
            positionedRows.rows,
            gridScrollMetrics.scrollTop,
            gridScrollMetrics.viewportHeight
        ]
    );

    return {
        densityConfig,
        gridGap,
        gridColumnCount,
        gridMinWidth,
        gridPadding,
        gridScrollRef,
        gridTotalHeight: positionedRows.totalHeight,
        visibleGridRows
    };
}
