import { useEffect, useState } from 'react';

type UseNowMsOptions = {
    active?: boolean;
    intervalMs?: number;
};

export function useNowMs({
    active = true,
    intervalMs = 1000
}: UseNowMsOptions = {}) {
    const [nowMs, setNowMs] = useState(Date.now);

    useEffect(() => {
        if (!active) {
            return undefined;
        }
        const intervalId = window.setInterval(
            () => setNowMs(Date.now()),
            intervalMs
        );
        return () => window.clearInterval(intervalId);
    }, [active, intervalMs]);

    return nowMs;
}
