import type { ComponentProps } from 'react';

import { EmptyState } from '@/components/layout/PageScaffold';
import { TileShell } from '@/components/tile/TileShell';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/ui/shadcn/skeleton';

import type { FavoritesDensityConfig } from '../favoritesDensity';

const FAVORITES_SKELETON_MAX_ROWS = 12;

function FavoritesEmptyState({
    title,
    description,
    className,
    ...props
}: ComponentProps<typeof EmptyState>) {
    return (
        <EmptyState
            {...props}
            variant="panel"
            title={title}
            description={description}
            className={cn('h-full min-h-60 border-0 p-6', className)}
        />
    );
}

function FavoritesSkeletonTile({
    densityConfig
}: {
    densityConfig: FavoritesDensityConfig;
}) {
    if (densityConfig.layout === 'cover') {
        return (
            <TileShell className="flex h-full w-full flex-col">
                <Skeleton
                    className="w-full shrink-0 rounded-none"
                    style={{
                        aspectRatio: String(densityConfig.imageAspectRatio)
                    }}
                />
                <div className="flex min-h-0 flex-1 flex-col justify-center gap-1.5 px-2.5 py-2">
                    <Skeleton className="h-3.5 w-3/5" />
                    <Skeleton className="h-3 w-2/5" />
                </div>
            </TileShell>
        );
    }

    return (
        <TileShell className="flex h-full w-full items-center gap-2.5 px-2.5">
            <Skeleton
                className="shrink-0 rounded-sm"
                style={{
                    width: `${densityConfig.mediaWidth}px`,
                    height: `${densityConfig.mediaHeight}px`
                }}
            />
            <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                <Skeleton className="h-3.5 w-3/5" />
                <Skeleton className="h-3 w-2/5" />
            </div>
        </TileShell>
    );
}

function FavoritesSkeletonGrid({
    cardHeight,
    columnCount,
    densityConfig,
    gridGap,
    gridMinWidth,
    gridPadding,
    viewportHeight
}: {
    cardHeight: number;
    columnCount: number;
    densityConfig: FavoritesDensityConfig;
    gridGap: number;
    gridMinWidth: number;
    gridPadding: number;
    viewportHeight: number;
}) {
    const cellHeight = cardHeight + gridPadding * 2;
    const rowCount = Math.min(
        FAVORITES_SKELETON_MAX_ROWS,
        Math.max(1, Math.ceil(viewportHeight / Math.max(1, cellHeight)))
    );

    return (
        <div
            aria-hidden
            className="grid min-w-0"
            style={{
                gap: `${gridGap}px`,
                gridTemplateColumns: `repeat(${columnCount}, minmax(${gridMinWidth}px, 1fr))`
            }}
        >
            {Array.from({ length: rowCount * columnCount }, (_, index) => (
                <div
                    key={`favorites-skeleton-${index}`}
                    className="min-h-0 min-w-0"
                    style={{
                        height: `${cardHeight}px`,
                        padding: `${gridPadding}px`
                    }}
                >
                    <FavoritesSkeletonTile densityConfig={densityConfig} />
                </div>
            ))}
        </div>
    );
}

export { FavoritesEmptyState, FavoritesSkeletonGrid };
