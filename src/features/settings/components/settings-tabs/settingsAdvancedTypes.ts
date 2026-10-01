import type { AvatarAutoCleanupPreference } from '@/shared/constants/settings';

export type SettingsAdvancedPrefs = {
    avatarAutoCleanup?: AvatarAutoCleanupPreference;
    feedPersistenceDisabled?: boolean;
    logResourceLoad?: boolean;
    udonExceptionLogging?: boolean;
};

export type SettingsAdvancedAction = () => void | Promise<void>;

export type SettingsAdvancedModel = {
    avatarAutoCleanupOptions: readonly AvatarAutoCleanupPreference[];
    onAvatarAutoCleanupChange: (value: AvatarAutoCleanupPreference) => void;
    onFeedPersistenceDisabledChange: (disabled: boolean) => void;
    onLogResourceLoadChange: (checked: boolean) => void;
    onOpenPurgeDialog: () => void;
    onRefreshOnlineVisits: SettingsAdvancedAction;
    onRefreshSqliteTableSizes: SettingsAdvancedAction;
    onUdonExceptionLoggingChange: (checked: boolean) => void;
    onlineVisitCount: number | null;
    prefs: SettingsAdvancedPrefs;
    sqliteTableSizeRows: ReadonlyArray<readonly [string, string]>;
    sqliteTableSizes: Record<string, unknown>;
};
