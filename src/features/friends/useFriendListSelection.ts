import { useEffect, useState } from 'react';

import {
    type FriendListRow,
    normalizeFriendListId as normalizeId
} from './friendListRows';

export function useFriendListSelection({
    filteredRows
}: {
    filteredRows: FriendListRow[];
}) {
    const [bulkUnfriendMode, setBulkUnfriendMode] = useState(false);
    const [selectedFriendIds, setSelectedFriendIds] = useState<Set<string>>(
        () => new Set<string>()
    );
    const [deletingFriendIds, setDeletingFriendIds] = useState<Set<string>>(
        () => new Set<string>()
    );
    const [isBulkDeleting, setIsBulkDeleting] = useState(false);

    useEffect(() => {
        if (!bulkUnfriendMode) {
            setSelectedFriendIds(new Set<string>());
        }
    }, [bulkUnfriendMode]);

    useEffect(() => {
        const visibleFriendIds = new Set(
            filteredRows
                .map((friend) => normalizeId(friend?.id))
                .filter(Boolean)
        );
        setSelectedFriendIds((current) => {
            const next = new Set(
                [...current].filter((friendId) =>
                    visibleFriendIds.has(friendId)
                )
            );
            return next.size === current.size ? current : next;
        });
    }, [filteredRows]);

    return {
        bulkUnfriendMode,
        deletingFriendIds,
        isBulkDeleting,
        selectedFriendIds,
        setBulkUnfriendMode,
        setDeletingFriendIds,
        setIsBulkDeleting,
        setSelectedFriendIds
    };
}
