import { useEffect } from 'react';

import {
    useCriticalTaskStore,
    type CriticalTaskId
} from '@/state/criticalTaskStore';

export function useCriticalTask(taskId: CriticalTaskId, active: boolean): void {
    useEffect(() => {
        if (!active) {
            return undefined;
        }
        const { setCriticalTaskActive } = useCriticalTaskStore.getState();
        setCriticalTaskActive(taskId, true);
        return () => setCriticalTaskActive(taskId, false);
    }, [taskId, active]);
}
