import { create } from 'zustand';

interface MutualGraphRevisionStoreState {
    revision: number;
    ownerUserId: string;
    bumpRevision(ownerUserId: string): void;
    reset(): void;
}

const initialState = {
    revision: 0,
    ownerUserId: ''
};

export const useMutualGraphRevisionStore =
    create<MutualGraphRevisionStoreState>((set) => ({
        ...initialState,
        bumpRevision(ownerUserId) {
            const normalizedOwnerUserId = ownerUserId.trim();
            if (!normalizedOwnerUserId) {
                return;
            }
            set((state) => ({
                revision:
                    state.ownerUserId === normalizedOwnerUserId
                        ? state.revision + 1
                        : 1,
                ownerUserId: normalizedOwnerUserId
            }));
        },
        reset() {
            set({ ...initialState });
        }
    }));
