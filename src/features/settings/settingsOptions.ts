import { AVATAR_AUTO_CLEANUP_OPTIONS } from '@/shared/constants/settings';

export const notificationLayoutOptions = [
    [
        'notification-center',
        'view.settings.notifications.notifications.layout_notification_center'
    ],
    ['table', 'view.settings.notifications.notifications.layout_table']
] as const;

export const avatarAutoCleanupOptions = AVATAR_AUTO_CLEANUP_OPTIONS;

export const sqliteTableSizeRows = [
    ['gps', 'view.settings.advanced.advanced.sqlite_table_size.gps'],
    ['status', 'view.settings.advanced.advanced.sqlite_table_size.status'],
    ['bio', 'view.settings.advanced.advanced.sqlite_table_size.bio'],
    ['avatar', 'view.settings.advanced.advanced.sqlite_table_size.avatar'],
    [
        'onlineOffline',
        'view.settings.advanced.advanced.sqlite_table_size.online_offline'
    ],
    [
        'friendLogHistory',
        'view.settings.advanced.advanced.sqlite_table_size.friend_log_history'
    ],
    [
        'notification',
        'view.settings.advanced.advanced.sqlite_table_size.notification'
    ],
    ['location', 'view.settings.advanced.advanced.sqlite_table_size.location'],
    [
        'joinLeave',
        'view.settings.advanced.advanced.sqlite_table_size.join_leave'
    ],
    [
        'portalSpawn',
        'view.settings.advanced.advanced.sqlite_table_size.portal_spawn'
    ],
    [
        'videoPlay',
        'view.settings.advanced.advanced.sqlite_table_size.video_play'
    ],
    ['event', 'view.settings.advanced.advanced.sqlite_table_size.event']
] as const;

export const settingsTabs = [
    ['system', 'view.settings.category.system'],
    ['interface', 'view.settings.category.interface'],
    ['social', 'view.settings.category.social'],
    ['notifications', 'view.settings.category.notifications'],
    ['media', 'view.settings.category.media'],
    ['ai', 'view.settings.category.ai'],
    ['advanced', 'view.settings.category.advanced']
];

export function resolveActiveSettingsTab(
    requestedTab: string,
    lastSettingsTab: string
): string {
    if (settingsTabs.some(([value]) => value === requestedTab)) {
        return requestedTab;
    }
    if (settingsTabs.some(([value]) => value === lastSettingsTab)) {
        return lastSettingsTab;
    }
    return 'system';
}
