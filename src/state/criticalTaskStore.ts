import { create } from 'zustand';

type CriticalTaskId =
    | 'databaseMaintenance'
    | 'databaseUpgrade'
    | 'dataDirMigration'
    | 'favoriteCollectionShare'
    | 'profileRestore';

type CriticalTaskStore = {
    activeTasks: CriticalTaskId[];
    setCriticalTaskActive(taskId: CriticalTaskId, active: boolean): void;
};

export const useCriticalTaskStore = create<CriticalTaskStore>((set) => ({
    activeTasks: [],
    setCriticalTaskActive(taskId, active) {
        set((state) => {
            if (state.activeTasks.includes(taskId) === active) {
                return state;
            }
            return {
                activeTasks: active
                    ? [...state.activeTasks, taskId]
                    : state.activeTasks.filter((id) => id !== taskId)
            };
        });
    }
}));

export function isCriticalTaskActive(): boolean {
    return useCriticalTaskStore.getState().activeTasks.length > 0;
}

export type { CriticalTaskId };
