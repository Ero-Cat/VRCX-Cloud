import type { FriendRecord } from '@/domain/friends/types';

import {
    isValidMutualFriendId,
    normalizeMutualFriendId
} from './mutualFriendsSettings';
import type {
    MutualFriendGraph,
    MutualFriendLink,
    MutualFriendMeta,
    MutualFriendNode,
    MutualFriendsCoverage,
    MutualFriendSnapshot,
    MutualFriendsGraphExtras
} from './mutualFriendsTypes';

export function mutualFriendUsername(friend: FriendRecord | null | undefined) {
    return typeof friend?.username === 'string' ? friend.username : '';
}

export function buildMutualFriendsCoverage(
    meta: MutualFriendMeta | null | undefined,
    friendIds: readonly string[] | null | undefined
): MutualFriendsCoverage {
    const metaMap = meta instanceof Map ? meta : new Map();
    const coverage: MutualFriendsCoverage = {
        friendCount: 0,
        fetchedCount: 0,
        unavailableCount: 0,
        lastFetchedAt: null
    };
    let latestFetchedTime = Number.NEGATIVE_INFINITY;

    for (const friendId of friendIds ?? []) {
        const normalizedId = normalizeMutualFriendId(friendId);
        if (!isValidMutualFriendId(normalizedId)) {
            continue;
        }
        coverage.friendCount += 1;

        const metadata = metaMap.get(normalizedId);
        if (!metadata) {
            continue;
        }
        if (metadata.optedOut) {
            coverage.unavailableCount += 1;
        }
        if (!metadata.lastFetchedAt) {
            continue;
        }
        coverage.fetchedCount += 1;
        const fetchedTime = Date.parse(metadata.lastFetchedAt);
        if (Number.isFinite(fetchedTime) && fetchedTime > latestFetchedTime) {
            latestFetchedTime = fetchedTime;
            coverage.lastFetchedAt = metadata.lastFetchedAt;
        }
    }

    return coverage;
}

export function buildMutualFriendsBaseGraph(
    snapshot: MutualFriendSnapshot | null | undefined,
    meta: MutualFriendMeta | null | undefined,
    friendLabelsById: Readonly<Record<string, string>> | null | undefined,
    excludedFriendIds: readonly string[] = [],
    extras: MutualFriendsGraphExtras | null | undefined = null
): MutualFriendGraph {
    const nodeMap = new Map<string, MutualFriendNode>();
    const totalCountById = new Map<string, number>();
    const edgeMap = new Map<string, MutualFriendLink>();
    const metaMap = meta instanceof Map ? meta : new Map();
    const excluded = new Set(
        excludedFriendIds.map(normalizeMutualFriendId).filter(Boolean)
    );

    const externalLabels = new Map(
        (extras?.externalUsers ?? []).map((user) => [
            normalizeMutualFriendId(user.id),
            user.displayName
        ])
    );

    function ensureNode(id: string, external = false): MutualFriendNode | null {
        const normalizedId = normalizeMutualFriendId(id);
        if (
            !isValidMutualFriendId(normalizedId) ||
            excluded.has(normalizedId)
        ) {
            return null;
        }
        const existing = nodeMap.get(normalizedId);
        if (existing) {
            return existing;
        }
        const metadata = metaMap.get(normalizedId);
        const isExternal = external || externalLabels.has(normalizedId);
        const node: MutualFriendNode = {
            id: normalizedId,
            label:
                friendLabelsById?.[normalizedId] ||
                externalLabels.get(normalizedId) ||
                normalizedId,
            lastFetchedAt: metadata?.lastFetchedAt ?? null,
            optedOut: Boolean(metadata?.optedOut),
            degree: 0,
            mutualCount: 0,
            external: isExternal || undefined
        };
        if (Number.isFinite(metadata?.totalCount)) {
            totalCountById.set(normalizedId, Number(metadata?.totalCount));
        }
        nodeMap.set(normalizedId, node);
        return node;
    }

    if (snapshot instanceof Map) {
        snapshot.forEach((mutualIds, friendId) => {
            const source = ensureNode(friendId);
            if (!source) {
                return;
            }
            for (const mutualId of Array.isArray(mutualIds) ? mutualIds : []) {
                const target = ensureNode(mutualId);
                if (!target || target.id === source.id) {
                    continue;
                }
                edgeMap.set([source.id, target.id].sort().join('__'), {
                    source: source.id,
                    target: target.id
                });
            }
        });
    }

    // User-drawn links and pinned non-friend nodes survive snapshot
    // refreshes; an existing API edge stays as-is (no duplicate manual
    // edge), and manual edges connect anything, including external nodes.
    if (Array.isArray(extras?.manualLinks)) {
        for (const link of extras.manualLinks) {
            const source = ensureNode(link.left, true);
            const target = ensureNode(link.right, true);
            if (!source || !target || source.id === target.id) {
                continue;
            }
            const key = [source.id, target.id].sort().join('__');
            if (!edgeMap.has(key)) {
                edgeMap.set(key, {
                    source: source.id,
                    target: target.id,
                    manual: true
                });
            }
        }
    }
    if (Array.isArray(extras?.externalUsers)) {
        for (const user of extras.externalUsers) {
            ensureNode(user.id, true);
        }
    }

    for (const edge of edgeMap.values()) {
        const source = nodeMap.get(edge.source);
        const target = nodeMap.get(edge.target);
        if (source) {
            source.degree += 1;
        }
        if (target) {
            target.degree += 1;
        }
    }

    for (const node of nodeMap.values()) {
        node.mutualCount = totalCountById.get(node.id) ?? node.degree;
    }

    return {
        nodes: Array.from(nodeMap.values()).sort(
            (left, right) => right.degree - left.degree
        ),
        links: Array.from(edgeMap.values())
    };
}
