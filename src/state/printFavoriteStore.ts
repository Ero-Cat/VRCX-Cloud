import { create } from 'zustand';

import type {
    CleanupWarning,
    PrintAutoCleanupEvent,
    PrintFavoriteState
} from '@/platform/tauri/bindings';
import {
    DEFAULT_PRINT_AUTO_DELETE_LIMIT,
    PRINT_FAVORITE_LIMIT_BUFFER
} from '@/shared/constants/settings';

const DEFAULT_MAX_FAVORITES =
    DEFAULT_PRINT_AUTO_DELETE_LIMIT - PRINT_FAVORITE_LIMIT_BUFFER;

type PrintFavoriteStoreState = {
    hydrated: boolean;
    lastCleanup: PrintAutoCleanupEvent | null;
    favoriteIds: string[];
    maxFavorites: number;
    warning: CleanupWarning | null;
    applyPrintCleanup(event: PrintAutoCleanupEvent): void;
    hydratePrintFavorites(state: PrintFavoriteState): void;
    removeFavoritePrintId(printId: string): void;
    resetPrintFavorites(): void;
};

export const usePrintFavoriteStore = create<PrintFavoriteStoreState>((set) => ({
    hydrated: false,
    lastCleanup: null,
    favoriteIds: [],
    maxFavorites: DEFAULT_MAX_FAVORITES,
    warning: null,
    applyPrintCleanup(event) {
        set({
            lastCleanup: event
        });
    },
    hydratePrintFavorites(state) {
        set({
            hydrated: true,
            favoriteIds: state.favoriteIds,
            maxFavorites: state.maxFavorites,
            warning: state.warning
        });
    },
    removeFavoritePrintId(printId) {
        const normalizedPrintId = printId.trim();
        if (!normalizedPrintId) {
            return;
        }
        set((current) => ({
            favoriteIds: current.favoriteIds.filter(
                (id) => id !== normalizedPrintId
            )
        }));
    },
    resetPrintFavorites() {
        set({
            hydrated: false,
            lastCleanup: null,
            favoriteIds: [],
            maxFavorites: DEFAULT_MAX_FAVORITES,
            warning: null
        });
    }
}));

export type { PrintFavoriteStoreState };
