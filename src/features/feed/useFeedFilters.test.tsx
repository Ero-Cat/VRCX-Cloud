// @vitest-environment jsdom

import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { useFeedFilters } from './useFeedFilters';

describe('useFeedFilters', () => {
    it('applies search only on commit and clears both the draft and query', () => {
        const { result } = renderHook(() => useFeedFilters());
        act(() => result.current.setSearchDraft('world'));
        expect(result.current.deferredSearchQuery).toBe('');
        act(() => result.current.commitSearch());
        expect(result.current.deferredSearchQuery).toBe('world');
        act(() => result.current.clearSearch());
        expect(result.current.searchDraft).toBe('');
        expect(result.current.deferredSearchQuery).toBe('');
    });

    it('applies and clears a date range independently of the search draft', () => {
        const { result } = renderHook(() => useFeedFilters());
        act(() => result.current.setSearchDraft('world'));
        act(() => result.current.setDateRange('2026-08-10', '2026-08-12'));
        expect(result.current.dateFrom).toBe('2026-08-10');
        expect(result.current.dateTo).toBe('2026-08-12');
        expect(result.current.deferredSearchQuery).toBe('');
        act(() => result.current.setDateRange('', ''));
        expect(result.current.searchDraft).toBe('world');
    });

    it('synchronizes the selected friends when the Feed route scope changes', async () => {
        const { result, rerender } = renderHook(
            ({ routeScopedUserIds }: { routeScopedUserIds: string[] }) =>
                useFeedFilters({ routeScopedUserIds }),
            {
                initialProps: {
                    routeScopedUserIds: ['usr_first']
                }
            }
        );

        expect(result.current.scopedUserIds).toEqual(['usr_first']);

        act(() => {
            rerender({ routeScopedUserIds: ['usr_second'] });
        });

        await waitFor(() => {
            expect(result.current.scopedUserIds).toEqual(['usr_second']);
        });
    });
});
