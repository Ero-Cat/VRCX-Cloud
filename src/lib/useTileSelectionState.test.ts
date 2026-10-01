// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import {
    computeSelectionRangeKeys,
    useTileSelectionState
} from './useTileSelectionState';

const keys = ['a', 'b', 'c', 'd', 'e'];

describe('computeSelectionRangeKeys', () => {
    it('selects the inclusive range regardless of click order', () => {
        expect(
            computeSelectionRangeKeys({ keys, fromIndex: 1, toIndex: 3 })
        ).toEqual(['b', 'c', 'd']);
        expect(
            computeSelectionRangeKeys({ keys, fromIndex: 3, toIndex: 1 })
        ).toEqual(['b', 'c', 'd']);
    });

    it('collapses to a single key when the range is a single index', () => {
        expect(
            computeSelectionRangeKeys({ keys, fromIndex: 2, toIndex: 2 })
        ).toEqual(['c']);
    });

    it('clamps out-of-range indexes to the bounds of the key list', () => {
        expect(
            computeSelectionRangeKeys({ keys, fromIndex: -5, toIndex: 2 })
        ).toEqual(['a', 'b', 'c']);
        expect(
            computeSelectionRangeKeys({ keys, fromIndex: 2, toIndex: 999 })
        ).toEqual(['c', 'd', 'e']);
    });

    it('returns an empty range for an empty key list', () => {
        expect(
            computeSelectionRangeKeys({ keys: [], fromIndex: 0, toIndex: 3 })
        ).toEqual([]);
    });
});

describe('useTileSelectionState', () => {
    it('extends the selection to a range on shift select', () => {
        const { result } = renderHook(() => useTileSelectionState({ keys }));

        act(() => {
            result.current.selectItem('b', true);
        });
        act(() => {
            result.current.selectItem('d', true, { shift: true });
        });

        expect([...result.current.selectedKeysSet]).toEqual(['b', 'c', 'd']);
        expect(result.current.hasSelection).toBe(true);
        expect(result.current.isAllSelected).toBe(false);
    });

    it('selects and clears every key through select all', () => {
        const { result } = renderHook(() => useTileSelectionState({ keys }));

        act(() => {
            result.current.toggleSelectAll();
        });
        expect(result.current.isAllSelected).toBe(true);

        act(() => {
            result.current.toggleSelectAll();
        });
        expect(result.current.selectedKeys).toEqual([]);
    });

    it('drops selected keys that disappear from the key list', () => {
        const { rerender, result } = renderHook(
            (props: { keys: string[] }) => useTileSelectionState(props),
            { initialProps: { keys: [...keys] } }
        );

        act(() => {
            result.current.selectItem('a', true);
            result.current.selectItem('e', true);
        });
        rerender({ keys: ['a', 'b'] });

        expect(result.current.selectedKeys).toEqual(['a']);
    });

    it('clears the selection when the reset token changes', () => {
        const { rerender, result } = renderHook(
            (props: { keys: string[]; resetToken: string }) =>
                useTileSelectionState(props),
            { initialProps: { keys, resetToken: 'prints' } }
        );

        act(() => {
            result.current.selectItem('a', true);
        });
        rerender({ keys, resetToken: 'gallery' });

        expect(result.current.selectedKeys).toEqual([]);
    });
});
