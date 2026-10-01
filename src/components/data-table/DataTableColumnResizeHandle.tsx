import type { RowData } from '@tanstack/react-table';
import { useRef, type KeyboardEvent, type PointerEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';

import type { AppHeader } from './appTable';

type ColumnResizeSession = {
    pointerId: number;
    startWidth: number;
    startX: number;
};

function clampColumnSize<TRow extends RowData>(
    header: AppHeader<TRow>,
    size: number
) {
    const minSize = header.column.columnDef.minSize ?? 20;
    const maxSize = header.column.columnDef.maxSize ?? Number.MAX_SAFE_INTEGER;
    return Math.min(maxSize, Math.max(minSize, Math.round(size)));
}

function resizeColumnFromKeyboard<TRow extends RowData>(
    event: KeyboardEvent<HTMLButtonElement>,
    header: AppHeader<TRow>
) {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') {
        return;
    }
    event.preventDefault();
    const direction = event.key === 'ArrowRight' ? 1 : -1;
    const step = event.shiftKey ? 32 : 16;
    const size = clampColumnSize(
        header,
        header.column.getSize() + direction * step
    );
    header.getContext().table.setColumnSizing((current) => ({
        ...current,
        [header.column.id]: size
    }));
}

export function DataTableColumnResizeHandle<TRow extends RowData>({
    header,
    label
}: {
    header: AppHeader<TRow>;
    label: string;
}) {
    const { t } = useTranslation();
    const minSize = header.column.columnDef.minSize ?? 20;
    const maxSize = header.column.columnDef.maxSize ?? Number.MAX_SAFE_INTEGER;
    const resizeSessionRef = useRef<ColumnResizeSession | null>(null);

    const updateResize = (event: PointerEvent<HTMLButtonElement>) => {
        const session = resizeSessionRef.current;
        if (!session || session.pointerId !== event.pointerId) {
            return;
        }
        const size = clampColumnSize(
            header,
            session.startWidth + event.clientX - session.startX
        );
        header.getContext().table.setColumnSizing((current) => ({
            ...current,
            [header.column.id]: size
        }));
    };

    const endResize = (event: PointerEvent<HTMLButtonElement>) => {
        if (resizeSessionRef.current?.pointerId !== event.pointerId) {
            return;
        }
        updateResize(event);
        resizeSessionRef.current = null;
        if (event.currentTarget.hasPointerCapture(event.pointerId)) {
            event.currentTarget.releasePointerCapture(event.pointerId);
        }
    };

    return (
        <Button
            type="button"
            variant="ghost"
            role="separator"
            aria-label={t('accessibility.resize_column', { column: label })}
            aria-orientation="vertical"
            aria-valuemin={minSize}
            aria-valuemax={maxSize}
            aria-valuenow={header.column.getSize()}
            data-resizing={header.column.getIsResizing() ? '' : undefined}
            className="vrcx-0-column-resize absolute top-0 right-0 h-full w-1.5 cursor-col-resize touch-none rounded-none border-0 bg-transparent p-0 hover:bg-transparent"
            onPointerDown={(event) => {
                event.preventDefault();
                event.currentTarget.setPointerCapture(event.pointerId);
                resizeSessionRef.current = {
                    pointerId: event.pointerId,
                    startWidth:
                        event.currentTarget.parentElement?.getBoundingClientRect()
                            .width || header.column.getSize(),
                    startX: event.clientX
                };
            }}
            onPointerMove={updateResize}
            onPointerUp={endResize}
            onPointerCancel={endResize}
            onKeyDown={(event) => resizeColumnFromKeyboard(event, header)}
        />
    );
}
