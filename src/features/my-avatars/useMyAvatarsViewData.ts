import { useMemo } from 'react';

import { collectMyAvatarTags, filterMyAvatars } from './myAvatarsFilters';
import type {
    MyAvatarRow,
    MyAvatarsPlatformFilter,
    MyAvatarsReleaseStatusFilter
} from './myAvatarsTypes';

export function useMyAvatarsViewData({
    avatars,
    deferredSearchQuery,
    platformFilter,
    releaseStatusFilter,
    tagFilters
}: {
    avatars: MyAvatarRow[];
    deferredSearchQuery: string;
    platformFilter: MyAvatarsPlatformFilter;
    releaseStatusFilter: MyAvatarsReleaseStatusFilter;
    tagFilters: Set<string>;
}) {
    const allTags = useMemo(() => collectMyAvatarTags(avatars), [avatars]);
    const filteredAvatars = useMemo(() => {
        return filterMyAvatars({
            avatars,
            searchQuery: deferredSearchQuery,
            platformFilter,
            releaseStatusFilter,
            tagFilters
        });
    }, [
        avatars,
        deferredSearchQuery,
        platformFilter,
        releaseStatusFilter,
        tagFilters
    ]);
    const activeFilterCount =
        (releaseStatusFilter !== 'all' ? 1 : 0) +
        (platformFilter !== 'all' ? 1 : 0) +
        tagFilters.size;

    return {
        activeFilterCount,
        allTags,
        filteredAvatars
    };
}
