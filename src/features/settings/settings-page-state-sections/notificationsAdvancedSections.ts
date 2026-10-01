import {
    avatarAutoCleanupOptions,
    sqliteTableSizeRows
} from '../settingsOptions';
import type { SettingsSectionInput } from '../settingsPageStateSectionTypes';

type NotificationsSectionInput = SettingsSectionInput<
    | 'setWebhookNotificationsDialogOpen'
    | 'setPrefs'
    | 'saveStringPreference'
    | 'saveBoolPreference'
>;

type AdvancedSectionInput = SettingsSectionInput<
    | 'sqliteTableSizes'
    | 'onlineVisitCount'
    | 'saveBoolPreference'
    | 'handleFeedPersistenceDisabledChange'
    | 'saveStringPreference'
    | 'setPurgeDialogOpen'
    | 'refreshSqliteTableSizes'
    | 'refreshOnlineVisits'
>;

export function buildNotificationsSection({
    setWebhookNotificationsDialogOpen,
    setPrefs,
    saveStringPreference,
    saveBoolPreference
}: NotificationsSectionInput) {
    return {
        setWebhookNotificationsDialogOpen,
        setPrefs,
        saveStringPreference,
        saveBoolPreference
    };
}

export function buildAdvancedSection({
    sqliteTableSizes,
    onlineVisitCount,
    saveBoolPreference,
    handleFeedPersistenceDisabledChange,
    saveStringPreference,
    setPurgeDialogOpen,
    refreshSqliteTableSizes,
    refreshOnlineVisits
}: AdvancedSectionInput) {
    return {
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        saveBoolPreference,
        handleFeedPersistenceDisabledChange,
        saveStringPreference,
        setPurgeDialogOpen,
        refreshSqliteTableSizes,
        refreshOnlineVisits
    };
}
