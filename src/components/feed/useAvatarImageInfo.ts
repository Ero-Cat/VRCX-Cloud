import { useEffect, useState } from 'react';

import type { LoadStatus } from '@/domain/shared/types';
import { commands } from '@/platform/tauri/bindings';
import { useRuntimeStore } from '@/state/runtimeStore';

import { normalizeFeedId as normalizeId } from './feedRows';

export type AvatarImageInfo = {
    avatarName: string;
    ownerId: string;
    status: LoadStatus;
    cacheKey: string;
};

type AvatarImageInfoInput = {
    avatarName?: string | null;
    imageUrl?: string | null;
    ownerId?: string | null;
};

function cacheKeyFor(imageUrl: string, endpoint: string): string {
    return imageUrl ? `${endpoint.trim()}\n${imageUrl}` : '';
}

function infoState({
    avatarName = '',
    ownerId = '',
    status = 'idle',
    cacheKey = ''
}: Partial<AvatarImageInfo>): AvatarImageInfo {
    return {
        avatarName: avatarName.trim(),
        ownerId: normalizeId(ownerId),
        status,
        cacheKey
    };
}

function isSameInfo(left: AvatarImageInfo, right: AvatarImageInfo): boolean {
    return (
        left.avatarName === right.avatarName &&
        left.ownerId === right.ownerId &&
        left.status === right.status &&
        left.cacheKey === right.cacheKey
    );
}

function initialState(
    { avatarName, imageUrl, ownerId }: AvatarImageInfoInput,
    endpoint: string
): AvatarImageInfo {
    const hintedName = avatarName?.trim() ?? '';
    const hintedOwnerId = normalizeId(ownerId);
    const cacheKey = cacheKeyFor(imageUrl?.trim() ?? '', endpoint);
    if (!cacheKey) {
        return infoState({
            avatarName: hintedName,
            ownerId: hintedOwnerId,
            cacheKey
        });
    }
    if (hintedName || hintedOwnerId) {
        return infoState({
            avatarName: hintedName,
            ownerId: hintedOwnerId,
            status: 'ready',
            cacheKey
        });
    }
    return infoState({ status: 'running', cacheKey });
}

export function useAvatarImageInfo({
    avatarName,
    imageUrl,
    ownerId
}: AvatarImageInfoInput): AvatarImageInfo {
    const currentEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    const [info, setInfo] = useState(() =>
        initialState({ avatarName, imageUrl, ownerId }, currentEndpoint)
    );
    useEffect(() => {
        const update = (next: AvatarImageInfo) =>
            setInfo((current) => (isSameInfo(current, next) ? current : next));
        const hintedName = avatarName?.trim() ?? '';
        const hintedOwnerId = normalizeId(ownerId);
        const resolvedImageUrl = imageUrl?.trim() ?? '';
        const cacheKey = cacheKeyFor(resolvedImageUrl, currentEndpoint);

        if (!cacheKey || hintedName || hintedOwnerId) {
            update(
                infoState({
                    avatarName: hintedName,
                    ownerId: hintedOwnerId,
                    status: cacheKey ? 'ready' : 'idle',
                    cacheKey
                })
            );
            return undefined;
        }

        let active = true;
        setInfo((current) =>
            current.cacheKey === cacheKey && current.status === 'ready'
                ? current
                : infoState({ status: 'running', cacheKey })
        );
        commands
            .appFileMetadataGet(resolvedImageUrl)
            .then((file) => {
                if (active) {
                    update(
                        infoState({
                            avatarName: file?.avatarName ?? '',
                            ownerId: file?.ownerId ?? '',
                            status: 'ready',
                            cacheKey
                        })
                    );
                }
            })
            .catch(() => {
                if (active) {
                    update(infoState({ status: 'error', cacheKey }));
                }
            });
        return () => {
            active = false;
        };
    }, [avatarName, currentEndpoint, imageUrl, ownerId]);

    return info;
}
