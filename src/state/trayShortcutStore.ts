import { create } from 'zustand';

import type { TrayShortcutSnapshot } from '@/platform/tauri/bindings';

export const useTrayShortcutStore = create<{
    snapshot: TrayShortcutSnapshot | null;
}>(() => ({ snapshot: null }));
