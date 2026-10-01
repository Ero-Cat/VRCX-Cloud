import type { FriendProfileLoadStatus } from '@/state/runtimeStore';

const VISIBLE_FRIEND_PROFILE_LOAD_STATUSES = new Set<FriendProfileLoadStatus>([
    'running',
    'cancelling',
    'completed',
    'cancelled'
]);

export function isFriendProfileLoadStatusVisible(
    status: FriendProfileLoadStatus
): boolean {
    return VISIBLE_FRIEND_PROFILE_LOAD_STATUSES.has(status);
}
