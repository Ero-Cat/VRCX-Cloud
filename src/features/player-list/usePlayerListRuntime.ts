import { parseLocation } from '@/shared/utils/location';
import { normalizeString } from '@/shared/utils/string';
import { useRuntimeStore } from '@/state/runtimeStore';

export function usePlayerListRuntime() {
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentUserEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    const currentUserSnapshot = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const currentUserLocation = useRuntimeStore((state) => {
        return (
            state.gameState.currentLocation ||
            normalizeString(state.auth.currentUserSnapshot?.location) ||
            ''
        );
    });
    const currentUserWorldId = useRuntimeStore(
        (state) =>
            parseLocation(state.gameState.currentLocation || '').worldId ||
            normalizeString(state.auth.currentUserSnapshot?.worldId) ||
            ''
    );
    const currentLocationStartedAt = useRuntimeStore(
        (state) => state.gameState.currentLocationStartedAt
    );
    const isGameRunning = useRuntimeStore(
        (state) => state.gameState.isGameRunning === true
    );
    const addGameLogEventCount = useRuntimeStore(
        (state) => state.runtimeEvents.gameLogProjection.count
    );
    const gameLogTailSyncedAt = useRuntimeStore(
        (state) => state.updateLoop.lastGameLogSyncAt
    );

    return {
        addGameLogEventCount,
        currentLocationStartedAt,
        currentUserEndpoint,
        currentUserId,
        currentUserLocation,
        currentUserSnapshot,
        currentUserWorldId,
        gameLogTailSyncedAt,
        isGameRunning
    };
}
