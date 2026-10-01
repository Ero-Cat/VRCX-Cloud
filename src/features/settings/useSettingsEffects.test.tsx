// @vitest-environment jsdom

import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

vi.mock('@/services/themeService', () => ({
    normalizeZoomLevel: (value: unknown) => Number(value) || 100
}));

import { useSettingsEffects } from './useSettingsEffects';

describe('useSettingsEffects', () => {
    it('syncs the zoom input with the persisted zoom level', async () => {
        const setZoomInput = vi.fn();

        renderHook(() =>
            useSettingsEffects({
                setZoomInput,
                zoomLevel: 125
            })
        );

        await waitFor(() => {
            expect(setZoomInput).toHaveBeenCalledWith('125');
        });
    });
});
