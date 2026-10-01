import { commands, type RawJson } from '@/platform/tauri/bindings';
import { isRecord } from '@/shared/utils/record';
import { useFavoriteStore } from '@/state/favoriteStore';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useSessionStore } from '@/state/sessionStore';

import { syncStartupServicesTask } from './startupServicesStatus';

type FavoriteBootstrapOptions = {
    userId?: string;
    endpoint?: string;
    currentUserSnapshot?: RawJson;
};
type FavoriteBootstrapResult = {
    userId: string;
    stale: boolean;
    count: number;
};
type ActiveFavoriteHydration = {
    promise: Promise<FavoriteBootstrapResult>;
    invalidated: boolean;
};

const activeHydrations = new Map<string, ActiveFavoriteHydration>();

function normalizeUserId(value: unknown) {
    return typeof value === 'string'
        ? value.trim()
        : String(value ?? '').trim();
}

function getDisplayName(user: Record<string, unknown> | null | undefined) {
    return (
        normalizeUserId(user?.displayName) ||
        normalizeUserId(user?.username) ||
        normalizeUserId(user?.id)
    );
}

function favoriteBootstrapKey(userId: string, endpoint = '') {
    return `${userId}\u0000${endpoint}`;
}

function isCurrentFavoriteBootstrapTarget(userId: string, endpoint = '') {
    const runtimeState = useRuntimeStore.getState();
    const sessionState = useSessionStore.getState();

    return (
        runtimeState.auth.currentUserId === userId &&
        runtimeState.auth.currentUserEndpoint === endpoint &&
        sessionState.isLoggedIn &&
        sessionState.sessionPhase === 'ready'
    );
}

async function runFavoriteBootstrap({
    userId,
    endpoint = '',
    currentUserSnapshot,
    isCurrentTarget
}: FavoriteBootstrapOptions & {
    isCurrentTarget: () => boolean;
}): Promise<FavoriteBootstrapResult> {
    const currentSnapshot = isRecord(currentUserSnapshot)
        ? currentUserSnapshot
        : null;
    const normalizedUserId = normalizeUserId(userId || currentSnapshot?.id);
    if (!normalizedUserId) {
        throw new Error(
            'Favorites hydration requires an authenticated user id.'
        );
    }

    const displayName = getDisplayName(currentSnapshot) || normalizedUserId;
    const friendRosterById = useFriendRosterStore.getState().friendsById;

    useFavoriteStore
        .getState()
        .setFavoritesLoading(
            normalizedUserId,
            `Loading favorites baseline for ${displayName}.`
        );
    useSessionStore.getState().setFavoritesLoaded(false);
    useRuntimeStore
        .getState()
        .setStartupTask(
            'services',
            'running',
            `Loading favorites baseline for ${displayName}.`
        );

    const result = await commands.appSocialFavoritesBaselineGet({
        userId: normalizedUserId,
        endpoint,
        currentUserSnapshot: currentSnapshot,
        friendRosterById
    });
    const snapshot = result.snapshot;

    if (result.stale || !snapshot) {
        if (isCurrentTarget()) {
            throw new Error(
                `Favorites baseline was stale for ${normalizedUserId}.`
            );
        }

        return {
            userId: normalizedUserId,
            stale: true,
            count: result.count
        };
    }

    if (!isCurrentTarget()) {
        return {
            userId: normalizedUserId,
            stale: true,
            count: result.count
        };
    }

    useFavoriteStore.getState().setFavoritesSnapshot(snapshot);
    useSessionStore.getState().setFavoritesLoaded(true);
    syncStartupServicesTask([snapshot.detail]);

    return {
        userId: normalizedUserId,
        stale: false,
        count: result.count
    };
}

export function bootstrapFavorites(
    options: FavoriteBootstrapOptions
): Promise<FavoriteBootstrapResult> {
    const normalizedUserId = normalizeUserId(
        options.userId ||
            (isRecord(options.currentUserSnapshot)
                ? options.currentUserSnapshot.id
                : '')
    );
    const currentUserSnapshot = isRecord(options.currentUserSnapshot)
        ? options.currentUserSnapshot
        : null;

    if (!normalizedUserId || !currentUserSnapshot) {
        return Promise.reject(
            new Error('Favorites hydration requires an authenticated user id.')
        );
    }

    const activeKey = favoriteBootstrapKey(normalizedUserId, options.endpoint);
    const activeHydration = activeHydrations.get(activeKey);
    if (activeHydration && !activeHydration.invalidated) {
        return activeHydration.promise;
    }

    const hydration: ActiveFavoriteHydration = {
        promise: Promise.resolve({
            userId: normalizedUserId,
            stale: true,
            count: 0
        }),
        invalidated: false
    };
    const isCurrentTarget = () =>
        !hydration.invalidated &&
        isCurrentFavoriteBootstrapTarget(normalizedUserId, options.endpoint);
    const invalidateIfStale = () => {
        if (
            isCurrentFavoriteBootstrapTarget(normalizedUserId, options.endpoint)
        ) {
            return;
        }

        hydration.invalidated = true;
        if (activeHydrations.get(activeKey) === hydration) {
            activeHydrations.delete(activeKey);
        }
    };
    const unsubscribeSession = useSessionStore.subscribe(invalidateIfStale);
    const unsubscribeRuntime = useRuntimeStore.subscribe(invalidateIfStale);

    hydration.promise = runFavoriteBootstrap({
        ...options,
        userId: normalizedUserId,
        currentUserSnapshot,
        isCurrentTarget
    })
        .catch((error: unknown) => {
            if (isCurrentTarget()) {
                useRuntimeStore
                    .getState()
                    .setStartupTask(
                        'services',
                        'error',
                        error instanceof Error ? error.message : String(error)
                    );
                useFavoriteStore
                    .getState()
                    .setFavoritesError(
                        error instanceof Error ? error.message : String(error)
                    );
                useSessionStore.getState().setFavoritesLoaded(false);
            }

            throw error;
        })
        .finally(() => {
            unsubscribeSession();
            unsubscribeRuntime();
            if (activeHydrations.get(activeKey) === hydration) {
                activeHydrations.delete(activeKey);
            }
        });

    activeHydrations.set(activeKey, hydration);
    return hydration.promise;
}
