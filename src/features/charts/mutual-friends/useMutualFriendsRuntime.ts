import { useMutualFriendLabels } from '@/lib/mutual-friends/useMutualFriendLabels';
import { getResolvedThemeMode } from '@/services/themeService';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useShellStore } from '@/state/shellStore';

export function useMutualFriendsRuntime() {
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const friendLabelsById = useMutualFriendLabels();
    const orderedFriendIds = useFriendRosterStore(
        (state) => state.orderedFriendIds
    );
    const shellThemeMode = useShellStore((state) => state.themeMode);
    const resolvedTheme = getResolvedThemeMode(shellThemeMode);

    return {
        currentUserId: currentUserId ?? '',
        friendsById,
        friendLabelsById,
        orderedFriendIds,
        resolvedTheme
    };
}
