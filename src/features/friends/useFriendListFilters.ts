import { useEffect, useState } from 'react';

import { useSessionStore } from '@/state/sessionStore';

export function useFriendListFilters() {
    const isFavoritesLoaded = useSessionStore(
        (state) => state.isFavoritesLoaded
    );
    const [searchQuery, setSearchQuery] = useState('');
    const [favoritesOnly, setFavoritesOnly] = useState(false);
    const [activeSearchFilterIds, setActiveSearchFilterIds] = useState<
        Set<string>
    >(() => new Set<string>());

    useEffect(() => {
        if (!isFavoritesLoaded && favoritesOnly) {
            setFavoritesOnly(false);
        }
    }, [favoritesOnly, isFavoritesLoaded]);

    return {
        activeSearchFilterIds,
        favoritesOnly,
        isFavoritesLoaded,
        searchQuery,
        setActiveSearchFilterIds,
        setFavoritesOnly,
        setSearchQuery
    };
}
