export const FRIEND_LOG_TYPES = [
    'Friend',
    'Unfriend',
    'FriendRequest',
    'CancelFriendRequest',
    'DisplayName',
    'TrustLevel'
] as const;

export type FriendLogType = (typeof FRIEND_LOG_TYPES)[number];

export function isFriendLogType(value: unknown): value is FriendLogType {
    return FRIEND_LOG_TYPES.some((type) => type === value);
}
