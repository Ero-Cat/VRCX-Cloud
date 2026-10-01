import { useEffect } from 'react';

import { restoreNormalWindowModeForIntent } from '@/services/windowModeService';
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
        restoreNormalWindowModeForIntent();
        return () => setCriticalTaskActive(taskId, false);
    }, [taskId, active]);
}
