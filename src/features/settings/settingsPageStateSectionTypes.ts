import type { Dispatch, SetStateAction } from 'react';

import type { NotificationLayout, TableDensity } from '@/state/shellStore';

import type { createDefaultSettingsPrefs } from './settingsDefaultPrefs';
import type { FavoriteFriendGroupOption } from './settingsFavoriteGroupOptions';
import type { useSettingsActions } from './useSettingsActions';

type SettingsPagePrefs = ReturnType<typeof createDefaultSettingsPrefs> &
    Record<string, unknown>;
type SettingsPrefs = SettingsPagePrefs;
type SettingsAction = () => void;
type SettingsRollback = () => void;
type SettingsOptimisticUpdate = () => void | SettingsRollback;
type SettingsCallback<Args extends unknown[] = unknown[], Result = void> = {
    bivarianceHack(...args: Args): Result;
}['bivarianceHack'];
type SetSettingsPrefs = SettingsCallback<
    [
        | SettingsPrefs
        | ((current: SettingsPrefs) => SettingsPrefs | Record<string, unknown>)
    ]
>;

type DialogSectionInput = Pick<
    SettingsActionsState,
    | 'purgeAvatarFeedData'
    | 'saveTableLimitsDialog'
    | 'saveWebhookActivityFilters'
    | 'searchLimitError'
    | 'tableLimitsSaveDisabled'
    | 'tableMaxSizeError'
> & {
    purgeDialogOpen: boolean;
    purgeInProgress: boolean;
    purgePeriod: string;
    setPurgePeriod: (value: string) => void;
    setTableLimitsDialogOpen: (value: boolean) => void;
    setTableLimitsDraft: Dispatch<
        SetStateAction<{ maxTableSize: string; searchLimit: string }>
    >;
    setTablePageSizesDialogOpen: (value: boolean) => void;
    tableLimitsDialogOpen: boolean;
    tableLimitsDraft: { maxTableSize: string; searchLimit: string };
    tablePageSizesDialogOpen: boolean;
};

type SettingsActionsState = ReturnType<typeof useSettingsActions>;

export type BuildSettingsPageStateSectionsInput = DialogSectionInput & {
    activeSettingsTab: string;
    commit: (
        action: SettingsAction,
        optimistic?: SettingsOptimisticUpdate
    ) => Promise<boolean>;
    handleCropInstancePrintsChange: SettingsCallback<[boolean]>;
    handleFeedPersistenceDisabledChange: SettingsCallback<[boolean]>;
    locale: string;
    normalizeRecentActionCooldownMinutes: (value: string) => number;
    onlineVisitCount: number | null;
    openTableLimitsDialog: SettingsCallback;
    openTablePageSizesDialog: SettingsCallback;
    openUgcFolderSelector: SettingsCallback;
    prefs: SettingsPrefs;
    refreshOnlineVisits: () => void;
    refreshSqliteTableSizes: () => void;
    resetTrustColors: SettingsCallback;
    resetUgcFolder: SettingsCallback;
    saveBoolPreference: SettingsCallback<[string, string, boolean]>;
    saveFontFamilyPreference: SettingsCallback<[string]>;
    saveInterfaceZoomLevel: SettingsCallback<[string | number]>;
    savePreferenceValue: SettingsCallback<
        [string, unknown, SettingsAction],
        Promise<boolean>
    >;
    saveStringPreference: SettingsCallback<[string, string, string]>;
    saveTrustColor: SettingsCallback<[string, string]>;
    selectCjkFontPack: SettingsCallback<[string]>;
    setAccessibleStatusIndicatorsPreference: SettingsCallback<[boolean]>;
    setActiveSettingsTab: SettingsCallback<[string]>;
    setAppLanguagePreference: SettingsCallback<[string | null]>;
    setDataTableStripedPreference: SettingsCallback<[boolean]>;
    setIntConfigPreference: SettingsCallback<
        [string, number, { min?: number; max?: number; fallback?: number }]
    >;
    setNotificationLayoutPreference: SettingsCallback<[NotificationLayout]>;
    setPrefs: SetSettingsPrefs;
    setPurgeDialogOpen: SettingsCallback<[boolean]>;
    setProxyEnabledPreference: SettingsCallback<[boolean]>;
    setRecentActionCooldownEnabledPreference: SettingsCallback<[boolean]>;
    setRecentActionCooldownMinutesPreference: SettingsCallback<[number]>;
    setSaveInstanceEmojiPreference: SettingsCallback<[boolean]>;
    setSaveInstancePrintsPreference: SettingsCallback<[boolean]>;
    setSaveInstanceStickersPreference: SettingsCallback<[boolean]>;
    setShowNewDashboardButtonPreference: SettingsCallback<[boolean]>;
    setTableDensityPreference: SettingsCallback<[TableDensity]>;
    setWebhookNotificationsDialogOpen: SettingsCallback<[boolean]>;
    setZoomInput: SettingsCallback<[string]>;
    setZoomLevelPreference: SettingsCallback<[string | number]>;
    sqliteTableSizes: Record<string, unknown>;
    toggleLocalFavoriteFriendsGroup: SettingsCallback<[string, boolean]>;
    webhookNotificationsDialogOpen: boolean;
    zoomInput: string;
    zoomLevel: number | null;
    addFeedHiddenUser: SettingsCallback<[string]>;
    favoriteFriendGroupOptions: FavoriteFriendGroupOption[];
    localFavoriteFriendGroupOptions: FavoriteFriendGroupOption[];
    localFavoriteFriendsGroups: string[];
    remoteFavoriteFriendGroupOptions: FavoriteFriendGroupOption[];
    removeFeedHiddenUser: SettingsCallback<[string]>;
    selectedFavoriteFriendGroupLabel: string;
};

export type SettingsSectionInput<
    Keys extends keyof BuildSettingsPageStateSectionsInput
> = Pick<BuildSettingsPageStateSectionsInput, Keys>;
