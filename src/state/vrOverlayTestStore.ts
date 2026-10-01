import { create } from 'zustand';

import { commands } from '@/platform/tauri/bindings';

interface VrOverlayTestState {
    testMode: boolean;
    pending: boolean;
    setTestMode: (testMode: boolean) => Promise<void>;
}

export const useVrOverlayTestStore = create<VrOverlayTestState>((set, get) => ({
    testMode: false,
    pending: false,
    async setTestMode(testMode) {
        if (get().pending) {
            return;
        }
        set({ pending: true });
        try {
            const snapshot = await commands.appVrOverlayTestModeSet(testMode);
            set({ testMode: snapshot.testMode });
        } catch (error) {
            console.warn('Failed to set VR overlay test mode:', error);
        } finally {
            set({ pending: false });
        }
    }
}));

export type { VrOverlayTestState };
