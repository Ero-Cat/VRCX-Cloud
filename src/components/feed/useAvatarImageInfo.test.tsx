// @vitest-environment jsdom

import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    appFileMetadataGet: vi.fn()
}));

vi.mock('@/platform/tauri/bindings', () => ({
    commands: { appFileMetadataGet: mocks.appFileMetadataGet }
}));

vi.mock('@/state/runtimeStore', () => ({
    useRuntimeStore: <T,>(
        selector: (state: { auth: { currentUserEndpoint: string } }) => T
    ): T => selector({ auth: { currentUserEndpoint: '' } })
}));

import { useAvatarImageInfo } from './useAvatarImageInfo';

const imageUrl = 'https://api.vrchat.cloud/api/1/image/file_avatar/1/256';

beforeEach(() => {
    vi.resetAllMocks();
});

describe('useAvatarImageInfo', () => {
    it('resolves the avatar behind an image url through the file cache', async () => {
        mocks.appFileMetadataGet.mockResolvedValue({
            id: 'file_avatar',
            name: 'Avatar - Neko - Image - 1',
            ownerId: 'usr_author',
            avatarName: 'Neko'
        });

        const { result } = renderHook(() => useAvatarImageInfo({ imageUrl }));

        expect(result.current.status).toBe('running');
        await waitFor(() => {
            expect(result.current).toMatchObject({
                avatarName: 'Neko',
                ownerId: 'usr_author',
                status: 'ready'
            });
        });
        expect(mocks.appFileMetadataGet).toHaveBeenCalledWith(imageUrl);
    });

    it('reports custom icons as ready without an avatar name', async () => {
        mocks.appFileMetadataGet.mockResolvedValue({
            id: 'file_icon',
            name: 'file_icon_camera_user_icon',
            ownerId: 'usr_self',
            avatarName: null
        });

        const { result } = renderHook(() => useAvatarImageInfo({ imageUrl }));

        await waitFor(() => {
            expect(result.current).toMatchObject({
                avatarName: '',
                ownerId: 'usr_self',
                status: 'ready'
            });
        });
    });

    it('keeps hints and skips the lookup when a name or owner is provided', () => {
        const { result } = renderHook(() =>
            useAvatarImageInfo({ imageUrl, avatarName: ' Hinted ' })
        );

        expect(result.current).toMatchObject({
            avatarName: 'Hinted',
            status: 'ready'
        });
        expect(mocks.appFileMetadataGet).not.toHaveBeenCalled();
    });

    it('stays idle without an image url', () => {
        const { result } = renderHook(() =>
            useAvatarImageInfo({ imageUrl: '' })
        );

        expect(result.current.status).toBe('idle');
        expect(mocks.appFileMetadataGet).not.toHaveBeenCalled();
    });
});
