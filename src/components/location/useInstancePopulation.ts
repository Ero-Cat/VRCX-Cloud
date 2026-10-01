import { useEffect, useRef, useState } from 'react';

import { normalizeInstanceCounts } from '@/components/user-hover-card/userHoverCardModel';
import vrchatInstanceRepository from '@/repositories/vrchatInstanceRepository';

type InstancePopulation = ReturnType<typeof normalizeInstanceCounts>;

export function useInstancePopulation({
    worldId,
    instanceId,
    enabled,
    refreshKey
}: {
    worldId: string;
    instanceId: string;
    enabled: boolean;
    refreshKey: string | number;
}) {
    const ref = useRef<HTMLDivElement>(null);
    const [visible, setVisible] = useState(false);
    const [population, setPopulation] = useState<InstancePopulation>(null);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        const node = ref.current;
        if (!node || !enabled || visible) {
            return undefined;
        }
        if (typeof IntersectionObserver !== 'function') {
            setVisible(true);
            return undefined;
        }
        const observer = new IntersectionObserver((entries) => {
            if (entries.some((entry) => entry.isIntersecting)) {
                setVisible(true);
                observer.disconnect();
            }
        });
        observer.observe(node);
        return () => {
            observer.disconnect();
        };
    }, [enabled, visible]);

    useEffect(() => {
        if (!enabled || !visible) {
            return undefined;
        }
        let active = true;
        setLoading(true);
        vrchatInstanceRepository
            .getInstance({ worldId, instanceId })
            .then((response) => {
                if (active) {
                    setPopulation(normalizeInstanceCounts(response.json));
                }
            })
            .catch(() => {})
            .finally(() => {
                if (active) {
                    setLoading(false);
                }
            });
        return () => {
            active = false;
        };
    }, [enabled, visible, worldId, instanceId, refreshKey]);

    return { ref, population, loading };
}
