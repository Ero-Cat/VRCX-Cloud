type KnownSizeRow = {
    height: number;
};

type VisibleKnownSizeRowsOptions<T extends KnownSizeRow & { top: number }> = {
    rows?: readonly T[] | null;
    scrollTop: number;
    viewportHeight: number;
    overscan: number;
};

export function positionKnownSizeRows<T extends KnownSizeRow>(
    rows: readonly T[] | null | undefined
) {
    let top = 0;
    const positionedRows = (rows ?? []).map((row) => {
        const height = Math.max(0, row.height);
        const positioned: T & { height: number; top: number } = {
            ...row,
            height,
            top
        };
        top += height;
        return positioned;
    });

    return {
        rows: positionedRows,
        totalHeight: top
    };
}

export function getVisibleKnownSizeRows<
    T extends KnownSizeRow & { top: number }
>({
    rows,
    scrollTop,
    viewportHeight,
    overscan
}: VisibleKnownSizeRowsOptions<T>) {
    const safeRows = rows ?? [];
    if (!safeRows.length) {
        return [];
    }

    const safeScrollTop = Math.max(0, scrollTop);
    const safeViewportHeight = Math.max(0, viewportHeight);
    const safeOverscan = Math.max(0, overscan);
    const start = Math.max(0, safeScrollTop - safeOverscan);
    const end = safeScrollTop + safeViewportHeight + safeOverscan;

    return safeRows.filter((row) => {
        const top = Math.max(0, row.top);
        const height = Math.max(0, row.height);
        return top + height >= start && top <= end;
    });
}
