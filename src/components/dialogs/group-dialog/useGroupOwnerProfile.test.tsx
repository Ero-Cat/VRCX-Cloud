// @vitest-environment jsdom

import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    getUserProfile: vi.fn()
}));

vi.mock('@/repositories/userProfileRepository', () => ({
    default: {
        getUserProfile: mocks.getUserProfile
    }
}));

import { useGroupOwnerProfile } from './useGroupOwnerProfile';

describe('useGroupOwnerProfile', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('uses the owner name from the group profile without another user request', () => {
        const { result } = renderHook(() =>
            useGroupOwnerProfile({
                currentEndpoint: 'https://api.example.test',
                friendsById: {},
                group: {
                    ownerId: 'usr_owner',
                    ownerDisplayName: 'Known owner'
                }
            })
        );

        expect(result.current).toBeNull();
        expect(mocks.getUserProfile).not.toHaveBeenCalled();
    });

    it('requests an unknown non-friend owner profile', async () => {
        mocks.getUserProfile.mockResolvedValue({
            id: 'usr_owner',
            displayName: 'Remote owner'
        });

        const { result } = renderHook(() =>
            useGroupOwnerProfile({
                currentEndpoint: 'https://api.example.test',
                friendsById: {},
                group: {
                    ownerId: 'usr_owner',
                    ownerDisplayName: ''
                }
            })
        );

        await waitFor(() => {
            expect(mocks.getUserProfile).toHaveBeenCalledWith({
                userId: 'usr_owner'
            });
            expect(result.current?.displayName).toBe('Remote owner');
        });
    });
});
