import { create } from 'zustand';

type HoverCardStore = {
    activeToken: number | null;
    claim(token: number): void;
    release(token: number): void;
};

let tokenCounter = 0;

export function nextHoverCardToken(): number {
    tokenCounter += 1;
    return tokenCounter;
}

export const useHoverCardStore = create<HoverCardStore>((set, get) => ({
    activeToken: null,
    claim(token: number) {
        set({ activeToken: token });
    },
    release(token: number) {
        if (get().activeToken === token) {
            set({ activeToken: null });
        }
    }
}));
