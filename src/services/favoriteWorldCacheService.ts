import { commands } from '@/platform/tauri/bindings';
import favoritePersistenceRepository from '@/repositories/favoritePersistenceRepository';
import { useFavoriteRevisionStore } from '@/state/favoriteRevisionStore';
import { useFavoriteStore } from '@/state/favoriteStore';

import {
    favoriteCachePayload,
    normalizeFavoriteCacheEntityId
} from './favoriteCachePayload';

export async function cacheWorldDetails(
    world: unknown,
    fallbackWorldId?: string
): Promise<boolean> {
    const entity = favoriteCachePayload(world);
    if (!entity) {
        return false;
    }
    const written = await commands.appFavoriteCacheSnapshot({
        kind: 'world',
        entity,
        fallbackEntityId: normalizeFavoriteCacheEntityId(fallbackWorldId)
    });
    if (written) {
        useFavoriteRevisionStore.getState().bumpWorldDetails();
    }
    return written;
}

async function isFavoriteWorldId(id: string): Promise<boolean> {
    const state = useFavoriteStore.getState();
    if (state.favoriteWorldIds.includes(id)) {
        return true;
    }
    const localFavorites =
        await favoritePersistenceRepository.getWorldFavorites();
    return localFavorites.some((row) => row.worldId === id);
}

export async function cacheFavoriteWorldDetails(
    world: unknown
): Promise<boolean> {
    const entity = favoriteCachePayload(world);
    if (!entity) {
        return false;
    }
    const id = normalizeFavoriteCacheEntityId(entity.id);
    if (!id || !(await isFavoriteWorldId(id))) {
        return false;
    }
    return cacheWorldDetails(entity);
}

function reportWorldCacheError(error: unknown): void {
    console.warn('Failed to cache favorite world details:', error);
}

export function persistWorldDetails(
    world: unknown,
    fallbackWorldId?: string
): void {
    void cacheWorldDetails(world, fallbackWorldId).catch(reportWorldCacheError);
}

export function persistFavoriteWorldDetails(world: unknown): void {
    void cacheFavoriteWorldDetails(world).catch(reportWorldCacheError);
}
