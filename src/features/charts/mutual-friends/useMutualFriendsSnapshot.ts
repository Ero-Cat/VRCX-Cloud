import { useEffect, useState } from 'react';

import type { MutualFriendsSnapshotStatus } from '@/lib/mutual-friends/mutualFriendsTypes';
import mutualGraphPersistenceRepository from '@/repositories/mutualGraphPersistenceRepository';

type MutualFriendsSnapshotData = Awaited<
    ReturnType<typeof mutualGraphPersistenceRepository.getSnapshot>
>;

function emptySnapshot(): MutualFriendsSnapshotData {
    return {
        snapshot: new Map(),
        meta: new Map(),
        manualLinks: [],
        externalUsers: []
    };
}

interface SnapshotOptions {
    currentUserId: string;
    currentUserIdRef: { current: string };
    reloadToken: number;
}

export function useMutualFriendsSnapshot({
    currentUserId,
    currentUserIdRef,
    reloadToken
}: SnapshotOptions) {
    const [status, setStatus] = useState<MutualFriendsSnapshotStatus>('idle');
    const [detail, setDetail] = useState('');
    const [snapshotData, setSnapshotData] =
        useState<MutualFriendsSnapshotData>(emptySnapshot());

    useEffect(() => {
        let active = true;

        if (!currentUserId) {
            setStatus('idle');
            setSnapshotData(emptySnapshot());
            return () => {
                active = false;
            };
        }

        setStatus('running');
        setDetail('');

        mutualGraphPersistenceRepository
            .getSnapshot(currentUserId)
            .then((result) => {
                if (!active) {
                    return;
                }

                setSnapshotData(result);
                setStatus('ready');
                setDetail('');
            })
            .catch((error: unknown) => {
                if (!active) {
                    return;
                }

                setStatus('error');
                setSnapshotData(emptySnapshot());
                setDetail(error instanceof Error ? error.message : '');
            });

        return () => {
            active = false;
        };
    }, [currentUserId, reloadToken]);

    async function reloadSnapshot(
        nextDetail: string,
        expectedUserId: string = currentUserId
    ) {
        if (!expectedUserId || currentUserIdRef.current !== expectedUserId) {
            return;
        }

        setStatus('running');
        try {
            const result =
                await mutualGraphPersistenceRepository.getSnapshot(
                    expectedUserId
                );
            if (currentUserIdRef.current !== expectedUserId) {
                return;
            }
            setSnapshotData(result);
            setStatus('ready');
            setDetail(nextDetail);
        } catch (error) {
            setSnapshotData(emptySnapshot());
            setStatus('error');
            setDetail(error instanceof Error ? error.message : '');
        }
    }

    return {
        detail,
        reloadSnapshot,
        setDetail,
        snapshotData,
        status
    };
}
