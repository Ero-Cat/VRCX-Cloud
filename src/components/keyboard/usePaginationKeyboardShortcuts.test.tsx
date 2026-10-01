// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useAppTable } from '@/components/data-table/appTable';
import { DataTablePagination } from '@/components/data-table/DataTableView';
import { SearchPagination } from '@/components/search/SearchPagination';

import { usePaginationKeyboardShortcuts } from './usePaginationKeyboardShortcuts';

function PaginationHarness({
    enabled = true,
    canPrevious = true,
    canNext = true,
    onPrevious = () => {},
    onNext = () => {}
}: {
    enabled?: boolean;
    canPrevious?: boolean;
    canNext?: boolean;
    onPrevious?(): void;
    onNext?(): void;
}) {
    const ref = usePaginationKeyboardShortcuts<HTMLDivElement>({
        enabled,
        canPrevious,
        canNext,
        onPrevious,
        onNext
    });
    return <div ref={ref} />;
}

function TablePaginationHarness() {
    const table = useAppTable({
        columns: [{ accessorKey: 'id' }],
        data: [{ id: 1 }, { id: 2 }, { id: 3 }],
        initialState: { pagination: { pageIndex: 0, pageSize: 1 } }
    });
    return <DataTablePagination table={table} />;
}

describe('pagination keyboard shortcuts', () => {
    beforeEach(() => {
        vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(
            function (this: HTMLElement) {
                const rects = this.closest(
                    '[hidden], [data-hidden], [inert], [aria-hidden="true"]'
                )
                    ? []
                    : [new DOMRect(0, 0, 100, 30)];
                return Object.assign(rects, {
                    item: (index: number) => rects[index] ?? null
                });
            }
        );
    });

    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
    });

    it('uses the current callbacks and page boundaries, and stops after disable or unmount', () => {
        const onPrevious = vi.fn();
        const onNext = vi.fn();
        const nextCallback = vi.fn();
        const { rerender, unmount } = render(
            <PaginationHarness onPrevious={onPrevious} onNext={onNext} />
        );
        fireEvent.keyDown(window, { key: 'ArrowLeft' });
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(onPrevious).toHaveBeenCalledTimes(1);
        expect(onNext).toHaveBeenCalledTimes(1);

        rerender(
            <PaginationHarness
                canPrevious={false}
                onPrevious={onPrevious}
                onNext={nextCallback}
            />
        );
        fireEvent.keyDown(window, { key: 'ArrowLeft' });
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(onPrevious).toHaveBeenCalledTimes(1);
        expect(onNext).toHaveBeenCalledTimes(1);
        expect(nextCallback).toHaveBeenCalledTimes(1);

        rerender(<PaginationHarness enabled={false} onNext={nextCallback} />);
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(nextCallback).toHaveBeenCalledTimes(1);
        rerender(<PaginationHarness onNext={nextCallback} />);
        unmount();
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(nextCallback).toHaveBeenCalledTimes(1);
    });

    it.each([
        { altKey: true },
        { ctrlKey: true },
        { metaKey: true },
        { shiftKey: true },
        { repeat: true },
        { isComposing: true }
    ])('ignores modified, repeated and composing events: %o', (options) => {
        const onNext = vi.fn();
        render(<PaginationHarness onNext={onNext} />);
        fireEvent.keyDown(window, { key: 'ArrowRight', ...options });
        expect(onNext).not.toHaveBeenCalled();
    });

    it('respects events already handled by a child control', () => {
        const onNext = vi.fn();
        render(<PaginationHarness onNext={onNext} />);
        const event = new KeyboardEvent('keydown', {
            key: 'ArrowRight',
            cancelable: true
        });
        event.preventDefault();
        window.dispatchEvent(event);
        expect(onNext).not.toHaveBeenCalled();
    });

    it.each([
        'input',
        'textarea',
        'select',
        'editable',
        'combobox',
        'slider',
        'tablist',
        'tree',
        'grid'
    ])('ignores focus inside %s', (kind) => {
        const onNext = vi.fn();
        const control =
            kind === 'input' || kind === 'textarea' || kind === 'select'
                ? document.createElement(kind)
                : document.createElement('div');
        if (kind === 'editable') {
            control.setAttribute('contenteditable', 'true');
        } else if (control.tagName === 'DIV') {
            control.setAttribute('role', kind);
        }
        control.tabIndex = 0;
        const { container } = render(<PaginationHarness onNext={onNext} />);
        container.append(control);
        control.focus();
        fireEvent.keyDown(control, { key: 'ArrowRight' });
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(onNext).not.toHaveBeenCalled();
    });

    it('ignores nested editable targets even when the event target is not focused', () => {
        const onNext = vi.fn();
        render(
            <>
                <PaginationHarness onNext={onNext} />
                <div contentEditable suppressContentEditableWarning>
                    <span data-testid="editable-child">text</span>
                </div>
            </>
        );
        fireEvent.keyDown(screen.getByTestId('editable-child'), {
            key: 'ArrowRight'
        });
        expect(onNext).not.toHaveBeenCalled();
    });

    it('only handles the visible tab when inactive panels stay mounted', () => {
        const firstNext = vi.fn();
        const secondNext = vi.fn();
        const { rerender } = render(
            <>
                <div role="tabpanel" hidden>
                    <PaginationHarness onNext={firstNext} />
                </div>
                <div role="tabpanel">
                    <PaginationHarness onNext={secondNext} />
                </div>
            </>
        );
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(firstNext).not.toHaveBeenCalled();
        expect(secondNext).toHaveBeenCalledTimes(1);
        rerender(
            <>
                <div role="tabpanel">
                    <PaginationHarness onNext={firstNext} />
                </div>
                <div role="tabpanel" hidden>
                    <PaginationHarness onNext={secondNext} />
                </div>
            </>
        );
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(firstNext).toHaveBeenCalledTimes(1);
        expect(secondNext).toHaveBeenCalledTimes(1);
    });

    it('suspends background pagination while a dialog is open and resumes after close', () => {
        const backgroundNext = vi.fn();
        const dialogNext = vi.fn();
        const { rerender } = render(
            <>
                <PaginationHarness onNext={backgroundNext} />
                <div role="dialog">
                    <PaginationHarness onNext={dialogNext} />
                </div>
            </>
        );
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(backgroundNext).not.toHaveBeenCalled();
        expect(dialogNext).toHaveBeenCalledTimes(1);
        rerender(
            <>
                <PaginationHarness onNext={backgroundNext} />
                <div role="dialog" />
            </>
        );
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(backgroundNext).not.toHaveBeenCalled();
        rerender(<PaginationHarness onNext={backgroundNext} />);
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(backgroundNext).toHaveBeenCalledTimes(1);
    });

    it.each(['menu', 'listbox', 'popover'])(
        'suspends pagination while a %s is open',
        (kind) => {
            const onNext = vi.fn();
            render(
                <>
                    <PaginationHarness onNext={onNext} />
                    <div
                        role={kind === 'popover' ? undefined : kind}
                        data-slot={
                            kind === 'popover' ? 'popover-content' : undefined
                        }
                    />
                </>
            );
            fireEvent.keyDown(window, { key: 'ArrowRight' });
            expect(onNext).not.toHaveBeenCalled();
        }
    );

    it('does not paginate another page scope while its control has focus', () => {
        const onNext = vi.fn();
        render(
            <>
                <div className="vrcx-0-page-scaffold">
                    <PaginationHarness onNext={onNext} />
                </div>
                <button>Outside</button>
            </>
        );
        screen.getByRole('button').focus();
        fireEvent.keyDown(screen.getByRole('button'), { key: 'ArrowRight' });
        expect(onNext).not.toHaveBeenCalled();
    });

    it('registers search shortcuts only while pagination is shown, with plain arrow hints', () => {
        const onPrev = vi.fn();
        const onNext = vi.fn();
        const { rerender } = render(
            <SearchPagination
                show
                prevDisabled
                nextDisabled={false}
                onPrev={onPrev}
                onNext={onNext}
            />
        );
        expect(screen.queryByText('Alt')).toBeNull();
        expect(screen.getByText('←')).toBeTruthy();
        expect(screen.getByText('→')).toBeTruthy();
        fireEvent.keyDown(window, { key: 'ArrowLeft' });
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(onPrev).not.toHaveBeenCalled();
        expect(onNext).toHaveBeenCalledTimes(1);
        rerender(
            <SearchPagination show={false} onPrev={onPrev} onNext={onNext} />
        );
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(onNext).toHaveBeenCalledTimes(1);
    });

    it('advances the real table pagination and respects both boundaries', () => {
        render(<TablePaginationHarness />);
        expect(screen.getByText('1 / 3')).toBeTruthy();
        fireEvent.keyDown(window, { key: 'ArrowLeft' });
        expect(screen.getByText('1 / 3')).toBeTruthy();
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(screen.getByText('2 / 3')).toBeTruthy();
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        fireEvent.keyDown(window, { key: 'ArrowRight' });
        expect(screen.getByText('3 / 3')).toBeTruthy();
        fireEvent.keyDown(window, { key: 'ArrowLeft' });
        expect(screen.getByText('2 / 3')).toBeTruthy();
    });
});
