import { commands } from '@/platform/native/bindings';

type MutualGraphMeta = {
    lastFetchedAt: string | null;
    optedOut: boolean;
    totalCount: number | null;
};

export type MutualGraphManualLink = {
    left: string;
    right: string;
    note: string;
    createdAt: string;
};

export type MutualGraphExternalUser = {
    id: string;
    displayName: string;
    avatarUrl: string;
};

async function getSnapshot(userId: string): Promise<{
    snapshot: Map<string, string[]>;
    meta: Map<string, MutualGraphMeta>;
    manualLinks: MutualGraphManualLink[];
    externalUsers: MutualGraphExternalUser[];
}> {
    const {
        friendIds,
        links,
        meta: metaRows,
        manualLinks: manualLinkRows,
        externalUsers: externalUserRows
    } = await commands.appMutualGraphSnapshotGet(userId.trim());

    const snapshot = new Map<string, string[]>();
    const meta = new Map<string, MutualGraphMeta>();

    for (const friendId of friendIds) {
        if (friendId && !snapshot.has(friendId)) {
            snapshot.set(friendId, []);
        }
    }

    for (const row of links) {
        const friendId = row.friendId;
        const mutualId = row.mutualId;
        if (!friendId || !mutualId) {
            continue;
        }

        const mutualIds = snapshot.get(friendId) ?? [];
        mutualIds.push(mutualId);
        snapshot.set(friendId, mutualIds);
    }

    for (const row of metaRows) {
        const friendId = row.friendId;
        if (!friendId) {
            continue;
        }

        meta.set(friendId, {
            lastFetchedAt: row.lastFetchedAt || null,
            optedOut: row.optedOut,
            totalCount: row.totalCount
        });
    }

    const manualLinks: MutualGraphManualLink[] = [];
    for (const row of manualLinkRows ?? []) {
        const friendId = row.friendId;
        const mutualId = row.mutualId;
        if (!friendId || !mutualId) {
            continue;
        }
        const [left, right] = [friendId, mutualId].sort();
        manualLinks.push({
            left,
            right,
            note: row.note || '',
            createdAt: row.createdAt || ''
        });
    }

    const externalUsers: MutualGraphExternalUser[] = [];
    for (const row of externalUserRows ?? []) {
        if (!row.userId) {
            continue;
        }
        externalUsers.push({
            id: row.userId,
            displayName: row.displayName || '',
            avatarUrl: row.avatarUrl || ''
        });
    }

    return {
        snapshot,
        meta,
        manualLinks,
        externalUsers
    };
}

async function addManualLink(
    userId: string,
    friendId: string,
    mutualId: string
): Promise<void> {
    await commands.appMutualGraphManualLinkAdd({
        userId,
        friendId,
        mutualId
    });
}

async function removeManualLink(
    userId: string,
    friendId: string,
    mutualId: string
): Promise<void> {
    await commands.appMutualGraphManualLinkRemove({
        userId,
        friendId,
        mutualId
    });
}

async function addExternalUser(
    userId: string,
    targetUserId: string,
    displayName: string
): Promise<void> {
    await commands.appMutualGraphExternalUserAdd({
        userId,
        targetUserId,
        displayName
    });
}

async function removeExternalUser(
    userId: string,
    targetUserId: string
): Promise<void> {
    await commands.appMutualGraphExternalUserRemove({
        userId,
        targetUserId
    });
}

const mutualGraphPersistenceRepository = Object.freeze({
    getSnapshot,
    addManualLink,
    removeManualLink,
    addExternalUser,
    removeExternalUser
});

export default mutualGraphPersistenceRepository;
