import { useEffect, useRef, useState } from 'react';

type ValueThrottle<T> = {
    push: (value: T, intervalMs: number) => void;
    dispose: () => void;
};

export function createValueThrottle<T>(
    onEmit: (value: T) => void
): ValueThrottle<T> {
    let lastEmit = 0;
    let timeout: ReturnType<typeof setTimeout> | null = null;
    let latest: T;

    return {
        push(value, intervalMs) {
            latest = value;
            const elapsed = Date.now() - lastEmit;
            if (elapsed >= intervalMs) {
                lastEmit = Date.now();
                onEmit(value);
                return;
            }
            if (timeout !== null) {
                return;
            }
            timeout = setTimeout(() => {
                timeout = null;
                lastEmit = Date.now();
                onEmit(latest);
            }, intervalMs - elapsed);
        },
        dispose() {
            if (timeout !== null) {
                clearTimeout(timeout);
                timeout = null;
            }
        }
    };
}

export function useThrottledValue<T>(value: T, intervalMs: number): T {
    const [throttledValue, setThrottledValue] = useState(value);
    const throttleRef = useRef<ValueThrottle<T> | null>(null);
    if (throttleRef.current === null) {
        throttleRef.current = createValueThrottle<T>(setThrottledValue);
    }
    const throttle = throttleRef.current;

    useEffect(() => {
        throttle.push(value, intervalMs);
    }, [throttle, value, intervalMs]);

    useEffect(() => () => throttle.dispose(), [throttle]);

    return throttledValue;
}
