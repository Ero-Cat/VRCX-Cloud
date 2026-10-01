import { useFavoriteStore } from '@/state/favoriteStore';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useSessionStore } from '@/state/sessionStore';

export function useFriendsLocationsRosterState() {
    const isFavoritesLoaded = useSessionStore(
        (state) => state.isFavoritesLoaded
    );
    const rosterStatus = useFriendRosterStore((state) => state.loadStatus);
    const rosterDetail = useFriendRosterStore((state) => state.detail);
    const onlineIds = useFriendRosterStore((state) => state.onlineIds);
    const activeIds = useFriendRosterStore((state) => state.activeIds);
    const offlineIds = useFriendRosterStore((state) => state.offlineIds);
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const remoteFavoriteFriendIds = useFavoriteStore(
        (state) => state.favoriteFriendIds
    );
    const favoriteFriendGroups = useFavoriteStore(
        (state) => state.favoriteFriendGroups
    );
    const groupedFavoriteFriendIdsByGroupKey = useFavoriteStore(
        (state) => state.groupedFavoriteFriendIdsByGroupKey
    );
    const localFriendFavorites = useFavoriteStore(
        (state) => state.localFriendFavorites
    );
    const localFriendFavoriteGroups = useFavoriteStore(
        (state) => state.localFriendFavoriteGroups
    );

    return {
        activeIds,
        favoriteFriendGroups,
        friendsById,
        groupedFavoriteFriendIdsByGroupKey,
        isFavoritesLoaded,
        localFriendFavoriteGroups,
        localFriendFavorites,
        offlineIds,
        onlineIds,
        remoteFavoriteFriendIds,
        rosterDetail,
        rosterStatus
    };
}
