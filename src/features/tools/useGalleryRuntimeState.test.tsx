// @vitest-environment jsdom

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    getProfile: vi.fn(),
    auth: {
        currentUserId: 'usr_self',
        currentUserEndpoint: 'https://api.vrchat.cloud/api/1',
        currentUserSnapshot: {
            id: 'usr_self',
            userIcon: 'old snapshot icon',
            profilePicOverride: 'old snapshot banner',
            tags: []
        }
    }
}));

vi.mock('@/repositories/userProfileRepository', () => ({
    default: { getUserAppearanceProfile: mocks.getProfile }
}));
vi.mock('@/state/runtimeStore', () => ({
    useRuntimeStore: <T,>(
        selector: (state: { auth: typeof mocks.auth }) => T
    ) => selector({ auth: mocks.auth })
}));
vi.mock('@/state/modalStore', () => ({
    useModalStore: <T,>(
        selector: (state: { openImagePreview: () => void }) => T
    ) => selector({ openImagePreview: vi.fn() })
}));

import { useGalleryRuntimeState } from './useGalleryRuntimeState';

let client: QueryClient;
function renderState() {
    return renderHook(() => useGalleryRuntimeState(), {
        wrapper: ({ children }: { children: ReactNode }) => (
            <QueryClientProvider client={client}>
                {children}
            </QueryClientProvider>
        )
    });
}

describe('useGalleryRuntimeState', () => {
    beforeEach(() => {
        client = new QueryClient();
        mocks.getProfile.mockReset();
        mocks.auth.currentUserId = 'usr_self';
    });
    afterEach(() => client.clear());

    it('reads media from the self profile and keeps it when a refresh fails', async () => {
        mocks.getProfile.mockResolvedValue({
            id: 'usr_self',
            userIcon: 'profile icon',
            bannerCustomUrl: 'profile banner'
        });
        const { result } = renderState();
        expect(result.current.userIcon).toBe('');
        await waitFor(() =>
            expect(result.current.userIcon).toBe('profile icon')
        );
        expect(result.current.bannerCustomUrl).toBe('profile banner');
        expect(mocks.getProfile).toHaveBeenCalledWith({
            userId: 'usr_self',
            asSelf: true
        });
        mocks.getProfile.mockRejectedValue(new Error('refresh failed'));
        await act(async () => {
            await expect(result.current.refreshMediaProfile()).rejects.toThrow(
                'refresh failed'
            );
        });
        await waitFor(() =>
            expect(result.current.mediaProfileError).toBe('refresh failed')
        );
        expect(result.current.userIcon).toBe('profile icon');
        expect(result.current.bannerCustomUrl).toBe('profile banner');
    });

    it('does not display a previous account response after switching accounts', async () => {
        let resolveOld!: (profile: { userIcon: string }) => void;
        mocks.getProfile.mockImplementation(({ userId }: { userId: string }) =>
            userId === 'usr_self'
                ? new Promise((resolve) => {
                      resolveOld = resolve;
                  })
                : Promise.resolve({ id: 'usr_other', userIcon: 'other icon' })
        );
        const { result, rerender } = renderState();
        mocks.auth.currentUserId = 'usr_other';
        rerender();
        await waitFor(() => expect(result.current.userIcon).toBe('other icon'));
        await act(async () => resolveOld({ userIcon: 'stale icon' }));
        expect(result.current.userIcon).toBe('other icon');
        expect(result.current.bannerCustomUrl).toBe('');
    });
});
