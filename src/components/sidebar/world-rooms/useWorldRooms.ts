import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

import { queryKeys } from '@/lib/entityQueryCache';
import worldProfileRepository from '@/repositories/worldProfileRepository';
import { MINUTE_MS, SECOND_MS } from '@/shared/constants/time';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';

import { buildWorldRoomRows } from './worldRoomsModel';

const WORLD_ROOMS_REFRESH_INTERVAL_MS = 5 * MINUTE_MS;
const WORLD_ROOMS_OPEN_REFRESH_THROTTLE_MS = 60 * SECOND_MS;
const WORLD_ROOMS_CACHE_MS = 30 * MINUTE_MS;

export function useWorldRoomsQuery(worldId: string) {
    const currentEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    return useQuery({
        queryKey: queryKeys.worldRooms(worldId, currentEndpoint || ''),
        queryFn: () =>
            worldProfileRepository.getWorldProfile({ worldId, dialog: true }),
        staleTime: WORLD_ROOMS_OPEN_REFRESH_THROTTLE_MS,
        gcTime: WORLD_ROOMS_CACHE_MS,
        refetchInterval: WORLD_ROOMS_REFRESH_INTERVAL_MS,
        retry: false
    });
}

export function useWorldRooms(worldId: string) {
    const currentLocation = useRuntimeStore(
        (state) => state.gameState.currentLocation
    );
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const worldQuery = useWorldRoomsQuery(worldId);
    const world = worldQuery.data ?? null;
    const rooms = useMemo(
        () =>
            buildWorldRoomRows({
                worldId,
                world,
                friends: Object.values(friendsById || {}),
                currentLocation
            }),
        [currentLocation, friendsById, world, worldId]
    );

    return { worldQuery, world, rooms, friendsById };
}
