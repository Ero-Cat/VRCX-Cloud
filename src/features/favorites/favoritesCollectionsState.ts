import type {
    FavoriteGroupMap,
    FavoriteKind,
    FavoriteRecord
} from '@/domain/favorites/types';
import type { FavoriteStore } from '@/state/favoriteStore';

import { normalizeFavoriteEntityId as normalizeEntityId } from './favoritesItems';

const EMPTY_ARRAY: [] = [];
const EMPTY_OBJECT: Record<string, never> = {};

type FavoritesCollectionsStoreSlice = Pick<
    FavoriteStore,
    | 'detail'
    | 'lastLoadedAt'
    | 'favoriteAvatarGroups'
    | 'favoriteAvatarIds'
    | 'favoriteFriendGroups'
    | 'favoriteWorldGroups'
    | 'favoriteWorldIds'
    | 'favoritesSortOrder'
    | 'groupedFavoriteFriendIdsByGroupKey'
    | 'localAvatarFavoriteGroups'
    | 'localAvatarFavorites'
    | 'localFriendFavoriteGroups'
    | 'localFriendFavorites'
    | 'remoteFavoritesById'
> & {
    loadStatus: string;
};

function addNormalizedFavoriteIds(
    ids: Set<string>,
    idsByGroupKey: FavoriteGroupMap
) {
    for (const groupIds of Object.values(idsByGroupKey)) {
        for (const favoriteId of groupIds) {
            const normalizedId = normalizeEntityId(favoriteId);
            if (normalizedId) {
                ids.add(normalizedId);
            }
        }
    }
}

export function buildFavoriteFriendFactIds({
    groupedFavoriteFriendIdsByGroupKey = EMPTY_OBJECT,
    kind,
    localFriendFavorites = EMPTY_OBJECT
}: {
    groupedFavoriteFriendIdsByGroupKey?: FavoriteGroupMap;
    kind: FavoriteKind;
    localFriendFavorites?: FavoriteGroupMap;
}) {
    if (kind !== 'friend') {
        return [];
    }

    const ids = new Set<string>();
    addNormalizedFavoriteIds(ids, groupedFavoriteFriendIdsByGroupKey);
    addNormalizedFavoriteIds(ids, localFriendFavorites);
    return Array.from(ids);
}

export function buildFavoriteAvatarDetailIds({
    favoriteAvatarIds = EMPTY_ARRAY,
    kind,
    localAvatarFavorites = EMPTY_OBJECT
}: {
    favoriteAvatarIds?: string[];
    kind: FavoriteKind;
    localAvatarFavorites?: FavoriteGroupMap;
}) {
    if (kind !== 'avatar') {
        return [];
    }

    const ids = new Set<string>();
    for (const avatarId of favoriteAvatarIds) {
        const normalizedId = normalizeEntityId(avatarId);
        if (normalizedId) {
            ids.add(normalizedId);
        }
    }
    addNormalizedFavoriteIds(ids, localAvatarFavorites);
    return Array.from(ids);
}

export function buildFavoriteAvatarTags({
    kind,
    remoteFavoritesById = EMPTY_OBJECT
}: {
    kind: FavoriteKind;
    remoteFavoritesById?: Record<string, FavoriteRecord>;
}) {
    if (kind !== 'avatar') {
        return [];
    }

    return Array.from(
        new Set(
            Object.values(remoteFavoritesById)
                .filter((favorite) => favorite.type === 'avatar')
                .map((favorite) =>
                    Array.isArray(favorite.tags) &&
                    typeof favorite.tags[0] === 'string'
                        ? favorite.tags[0].trim()
                        : ''
                )
                .filter(Boolean)
        )
    );
}

export function buildFavoriteWorldGroupTags({
    kind,
    remoteFavoritesById = EMPTY_OBJECT,
    selectedGroupKey
}: {
    kind: FavoriteKind;
    remoteFavoritesById?: Record<string, FavoriteRecord>;
    selectedGroupKey: string;
}): string[] {
    if (kind !== 'world') {
        return [];
    }

    const normalizedGroupKey = normalizeEntityId(selectedGroupKey);
    const selected: string[] = [];
    const rest: string[] = [];
    const seen = new Set<string>();
    for (const favorite of Object.values(remoteFavoritesById)) {
        if (favorite.type !== 'world' && favorite.type !== 'vrcPlusWorld') {
            continue;
        }
        const tag =
            Array.isArray(favorite.tags) && typeof favorite.tags[0] === 'string'
                ? favorite.tags[0].trim()
                : '';
        if (!tag || seen.has(tag)) {
            continue;
        }
        seen.add(tag);
        if (normalizeEntityId(favorite.$groupKey) === normalizedGroupKey) {
            selected.push(tag);
        } else {
            rest.push(tag);
        }
    }
    return [...selected, ...rest];
}

export function buildFavoriteRemoteGroupEntityIds({
    groupKey,
    kind,
    remoteFavoritesById = EMPTY_OBJECT
}: {
    groupKey: string;
    kind: FavoriteKind;
    remoteFavoritesById?: Record<string, FavoriteRecord>;
}): string[] {
    const normalizedGroupKey = normalizeEntityId(groupKey);
    if (!normalizedGroupKey || kind === 'friend') {
        return [];
    }

    const ids = new Set<string>();
    for (const favorite of Object.values(remoteFavoritesById)) {
        const matchesKind =
            kind === 'avatar'
                ? favorite.type === 'avatar'
                : favorite.type === 'world' || favorite.type === 'vrcPlusWorld';
        if (
            !matchesKind ||
            normalizeEntityId(favorite.$groupKey) !== normalizedGroupKey
        ) {
            continue;
        }
        const favoriteId = normalizeEntityId(favorite.favoriteId);
        if (favoriteId) {
            ids.add(favoriteId);
        }
    }
    return Array.from(ids);
}

export function selectFavoritesCollectionsState(kind: FavoriteKind) {
    return (state: FavoritesCollectionsStoreSlice) => {
        const isFriend = kind === 'friend';
        const isAvatar = kind === 'avatar';
        const isWorld = kind === 'world';

        return {
            favoriteLoadStatus: state.loadStatus,
            favoriteDetail: state.detail,
            favoriteLastLoadedAt: state.lastLoadedAt,
            favoritesSortOrder: state.favoritesSortOrder,
            remoteFavoritesById:
                isAvatar || isWorld ? state.remoteFavoritesById : EMPTY_OBJECT,
            favoriteFriendGroups: isFriend
                ? state.favoriteFriendGroups
                : EMPTY_ARRAY,
            favoriteWorldGroups: isWorld
                ? state.favoriteWorldGroups
                : EMPTY_ARRAY,
            favoriteAvatarGroups: isAvatar
                ? state.favoriteAvatarGroups
                : EMPTY_ARRAY,
            groupedFavoriteFriendIdsByGroupKey: isFriend
                ? state.groupedFavoriteFriendIdsByGroupKey
                : EMPTY_OBJECT,
            localAvatarFavorites: isAvatar
                ? state.localAvatarFavorites
                : EMPTY_OBJECT,
            localFriendFavorites: isFriend
                ? state.localFriendFavorites
                : EMPTY_OBJECT,
            localAvatarFavoriteGroups: isAvatar
                ? state.localAvatarFavoriteGroups
                : EMPTY_ARRAY,
            localFriendFavoriteGroups: isFriend
                ? state.localFriendFavoriteGroups
                : EMPTY_ARRAY,
            favoriteWorldIds: isWorld ? state.favoriteWorldIds : EMPTY_ARRAY,
            favoriteAvatarIds: isAvatar ? state.favoriteAvatarIds : EMPTY_ARRAY
        };
    };
}
