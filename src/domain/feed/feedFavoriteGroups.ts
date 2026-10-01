import type { FavoriteGroup } from '@/domain/favorites/types';

export type FeedFavoriteGroupOption = {
    key: string;
    label: string;
};

function normalizeFeedId(value: string | undefined) {
    return (value ?? '').trim();
}

export function buildFeedFavoriteGroupOptions({
    favoriteFriendGroups,
    localFriendFavoriteGroups
}: {
    favoriteFriendGroups: readonly FavoriteGroup[];
    localFriendFavoriteGroups: readonly string[];
}): FeedFavoriteGroupOption[] {
    const options = new Map<string, FeedFavoriteGroupOption>();
    for (const group of favoriteFriendGroups) {
        const key = normalizeFeedId(group.key || group.name);
        if (key) {
            options.set(key, {
                key,
                label:
                    normalizeFeedId(group?.displayName || group?.name || key) ||
                    key
            });
        }
    }
    for (const groupName of localFriendFavoriteGroups) {
        const label = normalizeFeedId(groupName);
        if (label) {
            options.set(`local:${label}`, {
                key: `local:${label}`,
                label
            });
        }
    }
    return [...options.values()].sort((left, right) =>
        left.label.localeCompare(right.label)
    );
}
