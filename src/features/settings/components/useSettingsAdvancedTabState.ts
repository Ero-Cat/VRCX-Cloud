import { useShallow } from 'zustand/react/shallow';

import type { AvatarAutoCleanupPreference } from '@/shared/constants/settings';
import { usePreferencesStore } from '@/state/preferencesStore';

import { useSettingsPageSection } from '../SettingsPageStateContext';

export function useSettingsAdvancedTabState() {
    const advanced = useSettingsPageSection('advanced');
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            avatarAutoCleanup: state.avatarAutoCleanup,
            feedPersistenceDisabled: state.feedPersistenceDisabled,
            udonExceptionLogging: state.udonExceptionLogging,
            logResourceLoad: state.logResourceLoad
        }))
    );
    const {
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
    } = advanced;

    const advancedTab = {
        prefs,
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        onUdonExceptionLoggingChange: (checked: boolean) => {
            saveBoolPreference(
                'udonExceptionLogging',
                'VRCX_udonExceptionLogging',
                checked
            );
        },
        onLogResourceLoadChange: (checked: boolean) => {
            saveBoolPreference('logResourceLoad', 'logResourceLoad', checked);
        },
        onFeedPersistenceDisabledChange: (checked: boolean) => {
            handleFeedPersistenceDisabledChange(checked);
        },
        onAvatarAutoCleanupChange: (value: AvatarAutoCleanupPreference) => {
            saveStringPreference(
                'avatarAutoCleanup',
                'avatarAutoCleanup',
                value
            );
        },
        onOpenPurgeDialog: () => setPurgeDialogOpen(true),
        onRefreshSqliteTableSizes: refreshSqliteTableSizes,
        onRefreshOnlineVisits: refreshOnlineVisits
    };

    return advancedTab;
}
