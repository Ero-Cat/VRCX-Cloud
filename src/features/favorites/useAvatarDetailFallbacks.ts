import { useMemo } from 'react';

import type { FavoriteKind } from '@/domain/favorites/types';
import type { LoadStatus } from '@/domain/shared/types';
import avatarProfileRepository from '@/repositories/avatarProfileRepository';

import {
    type DetailMap,
    getRemoteEntityCacheFallbackIds,
    useRemoteEntityCacheFallbackLoader
} from './remoteEntityCacheFallbacks';

type AvatarDetailFallbackInput = {
    avatarIds: string[];
    kind: FavoriteKind;
    remoteEntityDetailsData?: DetailMap;
    remoteEntityDetailsStatus: LoadStatus;
};

const fetchAvatarById = (avatarId: string) =>
    avatarProfileRepository.getAvatarProfile({ avatarId });

export function getAvatarDetailFallbackIds({
    avatarIds,
    kind,
    remoteEntityDetailsData,
    remoteEntityDetailsStatus
}: AvatarDetailFallbackInput): string[] {
    return getRemoteEntityCacheFallbackIds({
        entityIds: avatarIds,
        detailSources: [remoteEntityDetailsData],
        isReady: kind === 'avatar' && remoteEntityDetailsStatus === 'ready'
    });
}

export function useAvatarDetailFallbacks({
    avatarIds,
    kind,
    remoteEntityDetailsData,
    remoteEntityDetailsStatus
}: AvatarDetailFallbackInput): DetailMap {
    const fallbackAvatarIds = useMemo(
        () =>
            getAvatarDetailFallbackIds({
                avatarIds,
                kind,
                remoteEntityDetailsData,
                remoteEntityDetailsStatus
            }),
        [avatarIds, kind, remoteEntityDetailsData, remoteEntityDetailsStatus]
    );

    return useRemoteEntityCacheFallbackLoader(
        fallbackAvatarIds,
        fetchAvatarById
    );
}
