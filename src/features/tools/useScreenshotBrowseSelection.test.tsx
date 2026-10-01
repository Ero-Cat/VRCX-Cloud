// @vitest-environment jsdom

import { act, cleanup, render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { useScreenshotBrowseSelection } from './useScreenshotBrowseSelection';

type HookValue = ReturnType<typeof useScreenshotBrowseSelection>;

function setup(initialKeys: string[]) {
    let value: HookValue | null = null;

    function Harness({ keys }: { keys: string[] }) {
        value = useScreenshotBrowseSelection(keys);
        return null;
    }

    const view = render(<Harness keys={initialKeys} />);
    return {
        get current() {
            return value!;
        },
        get selected() {
            return [...value!.selectedPaths].sort();
        },
        openFolder(keys: string[]) {
            view.rerender(<Harness keys={keys} />);
        }
    };
}

describe('useScreenshotBrowseSelection', () => {
    afterEach(cleanup);

    it('keeps what was selected in a folder after switching to another one', () => {
        const harness = setup(['a1', 'a2']);

        act(() => harness.current.selectItem('a1', true));
        expect(harness.selected).toEqual(['a1']);

        harness.openFolder(['b1', 'b2']);
        expect(harness.selected).toEqual(['a1']);

        act(() => harness.current.selectItem('b1', true));
        expect(harness.selected).toEqual(['a1', 'b1']);
    });

    it('scopes select all to the open folder and leaves other folders alone', () => {
        const harness = setup(['a1', 'a2']);
        act(() => harness.current.selectItem('a1', true));
        harness.openFolder(['b1', 'b2']);

        act(() => harness.current.toggleSelectAll());
        expect(harness.selected).toEqual(['a1', 'b1', 'b2']);
        expect(harness.current.isAllSelected).toBe(true);

        act(() => harness.current.toggleSelectAll());
        expect(harness.selected).toEqual(['a1']);
        expect(harness.current.hasSelection).toBe(true);
    });

    it('limits shift range selection to the open folder', () => {
        const harness = setup(['a1', 'a2', 'a3', 'a4']);

        act(() => harness.current.selectItem('a1', true));
        act(() => harness.current.selectItem('a3', true, { shift: true }));
        expect(harness.selected).toEqual(['a1', 'a2', 'a3']);

        harness.openFolder(['b1', 'b2', 'b3']);
        act(() => harness.current.selectItem('b3', true, { shift: true }));
        expect(harness.selected).toEqual(['a1', 'a2', 'a3', 'b3']);
    });

    it('drops deleted paths and clears everything on demand', () => {
        const harness = setup(['a1', 'a2']);
        act(() => harness.current.toggleSelectAll());
        harness.openFolder(['b1', 'b2']);
        act(() => harness.current.toggleSelectAll());
        expect(harness.selected).toEqual(['a1', 'a2', 'b1', 'b2']);

        act(() => harness.current.removePaths(['a1', 'b2']));
        expect(harness.selected).toEqual(['a2', 'b1']);

        act(() => harness.current.clearSelection());
        expect(harness.selected).toEqual([]);
        expect(harness.current.hasSelection).toBe(false);
    });
});
