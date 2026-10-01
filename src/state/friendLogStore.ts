import { create } from 'zustand';

interface FriendLogStoreState {
    // Bumped whenever the backend reports a friend-log change (friend add/remove). The friend-log page
    // controller subscribes to this to re-query history even while it is the active route, where
    // shellStore.notifyMenu is suppressed by isCurrentMenuRoute.
    revision: number;
    bumpRevision: () => void;
}

export const useFriendLogStore = create<FriendLogStoreState>((set) => ({
    revision: 0,
    bumpRevision() {
        set((state) => ({ revision: state.revision + 1 }));
    }
}));

export type { FriendLogStoreState };
