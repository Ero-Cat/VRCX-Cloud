import { useEffect, useRef, useState } from 'react';

import { commands } from '@/platform/tauri/bindings';
import worldProfileRepository from '@/repositories/worldProfileRepository';

import { resolveMissingEntities } from './resolveMissingEntities';

export type ActivityWorldSummary = {
    name: string;
    thumbnailUrl: string;
    imageUrl: string;
    authorName: string;
    description: string;
};

export function useActivityWorldNames(
    worldIds: string[]
): Map<string, ActivityWorldSummary> {
    const [summaries, setSummaries] = useState<
        Map<string, ActivityWorldSummary>
    >(new Map());
    const resolvedRef = useRef(new Set<string>());
    const worldIdsKey = worldIds.join(',');

    useEffect(() => {
        const ids = worldIdsKey ? worldIdsKey.split(',').filter(Boolean) : [];
        if (ids.length === 0) {
            return;
        }
        let active = true;

        const merge = (id: string, value: ActivityWorldSummary) => {
            resolvedRef.current.add(id);
            setSummaries((previous) => new Map(previous).set(id, value));
        };

        void commands
            .appWorldSummariesGet(ids)
            .then((localRows) => {
                if (!active) {
                    return;
                }
                const resolved = new Map<string, ActivityWorldSummary>();
                for (const id of ids) {
                    const row = localRows[id];
                    if (row?.name) {
                        resolved.set(id, {
                            name: row.name,
                            thumbnailUrl:
                                row.thumbnailImageUrl || row.imageUrl || '',
                            imageUrl:
                                row.imageUrl || row.thumbnailImageUrl || '',
                            authorName: row.authorName,
                            description: row.description
                        });
                    }
                }
                setSummaries((previous) => new Map([...previous, ...resolved]));
            })
            .catch(() => {});

        const pending = ids.filter((id) => !resolvedRef.current.has(id));
        void resolveMissingEntities({
            ids: pending,
            isActive: () => active,
            fetchOne: async (worldId) => {
                const profile = await worldProfileRepository.getWorldProfile({
                    worldId
                });
                return profile?.name
                    ? {
                          name: profile.name,
                          thumbnailUrl:
                              profile.thumbnailImageUrl ||
                              profile.imageUrl ||
                              '',
                          imageUrl:
                              profile.imageUrl ||
                              profile.thumbnailImageUrl ||
                              '',
                          authorName: profile.authorName,
                          description: profile.description
                      }
                    : null;
            },
            onResolved: merge
        });

        return () => {
            active = false;
        };
    }, [worldIdsKey]);

    return summaries;
}
