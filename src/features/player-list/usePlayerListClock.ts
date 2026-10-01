import { useEffect, useState } from 'react';

export function usePlayerListClock() {
    const [clockNow, setClockNow] = useState(() => Date.now());

    useEffect(() => {
        const timer = window.setInterval(() => {
            setClockNow(Date.now());
        }, 30000);

        return () => {
            window.clearInterval(timer);
        };
    }, []);

    return clockNow;
}
