// @vitest-environment jsdom
import { cleanup, renderHook, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({ roster: vi.fn(), instance: vi.fn() }));
vi.mock('@/services/currentInstanceRosterService', () => ({
    loadCurrentInstanceRoster: mocks.roster
}));
vi.mock('@/repositories/vrchatInstanceRepository', () => ({
    default: { getInstance: mocks.instance }
}));
vi.mock('@/repositories/groupProfileRepository', () => ({ default: {} }));
vi.mock('@/repositories/userProfileRepository', () => ({ default: {} }));
vi.mock('@/services/domainIngestionService', () => ({
    recordGameRuntimePresence: vi.fn(),
    recordLocationHintsFromInstances: vi.fn()
}));

import { useWorldDialogCurrentInstance } from './useWorldDialogCurrentInstance';

afterEach(cleanup);

it('shows the backend roster while remote instance metadata is still pending', async () => {
    const worldId = 'wrld_00000000-0000-0000-0000-000000000000';
    const location = worldId + ':1';
    mocks.instance.mockReturnValue(new Promise(() => {}));
    mocks.roster.mockResolvedValue({
        context: {
            location,
            createdAt: '2026-05-14T04:00:00Z',
            source: 'runtime',
            playerFactsKnown: true,
            worldId,
            worldName: 'World',
            playerCount: 1,
            time: 0,
            groupName: ''
        },
        players: [
            {
                id: 'usr_player',
                userId: 'usr_player',
                displayName: 'Player',
                joinedAt: '2026-05-14T04:00:10Z',
                joinedAtMs: 1000
            }
        ]
    });
    const { result } = renderHook(() =>
        useWorldDialogCurrentInstance({
            currentResolvedLocation: location,
            isInstanceLocation: true,
            normalizedWorldId: location,
            runtime: {
                currentEndpoint: '',
                currentLocationPlayers: [],
                currentLocationStartedAt: null,
                currentUserId: 'usr_self',
                currentUserSnapshot: null
            }
        })
    );
    await waitFor(() =>
        expect(result.current.playerSnapshot?.players[0]?.displayName).toBe(
            'Player'
        )
    );
});
