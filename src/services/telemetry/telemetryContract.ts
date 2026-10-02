export const TELEMETRY_ROUTE_KEYS = [
    'friends_locations',
    'game_log',
    'instance_history',
    'player_list',
    'search',
    'dashboard',
    'favorites_friends',
    'favorites_worlds',
    'favorites_avatars',
    'friend_log',
    'moderation',
    'my_avatars',
    'notification',
    'friend_list',
    'charts_mutual',
    'tools',
    'settings'
] as const;

export const TELEMETRY_TOOL_KEYS = [
    'gallery',
    'inventory',
    'profile-backup',
    'llm-endpoints',
    'presence-schedule',
    'presence-room-rules',
    'presence-invite-requests',
    'group-calendar',
    'group-moderation',
    'my-groups',
    'discord-names',
    'export-notes',
    'export-friend-list',
    'export-own-avatars',
    'edit-invite-message'
] as const;

export type TelemetryPageRouteKey = (typeof TELEMETRY_ROUTE_KEYS)[number];
