import { useMemo } from 'react';

import type { FavoriteKind } from '@/domain/favorites/types';
import { useTileSelectionState } from '@/lib/useTileSelectionState';

import type { FavoriteItem } from './favoritesTypes';

export function useFavoritesSelectionState({
    contentItems,
    kind
}: {
    contentItems: FavoriteItem[];
    kind: FavoriteKind;
}) {
    const contentKeys = useMemo(
        () => contentItems.map((item) => item.key),
        [contentItems]
    );
    const selection = useTileSelectionState({
        keys: contentKeys,
        resetToken: kind
    });
    const selectedContentItems = useMemo(
        () =>
            contentItems.filter((item) =>
                selection.selectedKeysSet.has(item.key)
            ),
        [contentItems, selection.selectedKeysSet]
    );
    const avatarSelectionActionsDisabled =
        kind === 'avatar' &&
        selectedContentItems.some((item) => item.source !== 'remote');

    return {
        ...selection,
        avatarSelectionActionsDisabled,
        selectedContentItems
    };
}
