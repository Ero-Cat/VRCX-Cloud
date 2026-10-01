import { useEffect, useRef, useState } from 'react';

const DURATION_MS = 900;

function easeOut(progress: number) {
    return 1 - (1 - progress) ** 3;
}

export function useCountUp(target: number, decimals = 0): number {
    const [value, setValue] = useState(target);
    const frameRef = useRef(0);

    useEffect(() => {
        if (
            window.matchMedia('(prefers-reduced-motion: reduce)').matches ||
            target <= 0
        ) {
            setValue(target);
            return;
        }
        const start = performance.now();
        const step = (now: number) => {
            const progress = Math.min((now - start) / DURATION_MS, 1);
            setValue(target * easeOut(progress));
            if (progress < 1) {
                frameRef.current = requestAnimationFrame(step);
            }
        };
        frameRef.current = requestAnimationFrame(step);
        return () => cancelAnimationFrame(frameRef.current);
    }, [target]);

    const factor = 10 ** decimals;
    return Math.round(value * factor) / factor;
}
