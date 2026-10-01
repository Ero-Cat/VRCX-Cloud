import { useEffect, useRef, useState } from 'react';

import { commands, type AvatarUsageRow } from '@/platform/tauri/bindings';
import avatarProfileRepository from '@/repositories/avatarProfileRepository';

import { resolveMissingEntities } from './resolveMissingEntities';

const AVATAR_LIMIT = 10;

export function useActivityAvatarUsage(
    ownerUserId: string,
    enabled: boolean
): AvatarUsageRow[] {
    const [rows, setRows] = useState<AvatarUsageRow[]>([]);
    const resolvedRef = useRef(new Set<string>());

    useEffect(() => {
        if (!ownerUserId || !enabled) {
            return;
        }
        let active = true;
        const isActive = () => active;

        void commands
            .appAvatarUsageRanking(ownerUserId, AVATAR_LIMIT)
            .then(async (ranking) => {
                if (!active) {
                    return;
                }
                setRows(ranking);

                const pending = ranking
                    .map((row) => row.avatarId)
                    .filter((id) => !resolvedRef.current.has(id));
                await resolveMissingEntities({
                    ids: pending,
                    isActive,
                    fetchOne: async (avatarId) => {
                        const profile =
                            await avatarProfileRepository.getAvatarProfile({
                                avatarId
                            });
                        return profile?.name ? profile : null;
                    },
                    onResolved: (avatarId, profile) => {
                        resolvedRef.current.add(avatarId);
                        setRows((previous) =>
                            previous.map((row) =>
                                row.avatarId === avatarId
                                    ? {
                                          ...row,
                                          name: profile.name,
                                          thumbnailImageUrl:
                                              profile.thumbnailImageUrl ||
                                              row.thumbnailImageUrl,
                                          imageUrl:
                                              profile.imageUrl || row.imageUrl
                                      }
                                    : row
                            )
                        );
                    }
                });
            })
            .catch(() => {});

        return () => {
            active = false;
        };
    }, [enabled, ownerUserId]);

    return rows;
}
