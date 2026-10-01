import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useShallow } from 'zustand/react/shallow';

import {
    setAccessibleStatusIndicatorsPreference,
    setAppLanguagePreference,
    setDataTableStripedPreference,
    setIntConfigPreference,
    setNotificationLayoutPreference,
    setRecentActionCooldownEnabledPreference,
    setRecentActionCooldownMinutesPreference,
    setShowNewDashboardButtonPreference,
    setSaveInstanceEmojiPreference,
    setSaveInstancePrintsPreference,
    setSaveInstanceStickersPreference,
    setTableDensityPreference,
    setZoomLevelPreference
} from '@/services/preferencesService';
import {
    DEFAULT_MAX_TABLE_SIZE,
    DEFAULT_SEARCH_LIMIT
} from '@/shared/constants/settings';
import { MINUTES_PER_DAY } from '@/shared/constants/time';
import { useFavoriteStore } from '@/state/favoriteStore';
import {
    DEFAULT_PREFERENCES,
    normalizePreferenceSnapshot,
    usePreferencesStore,
    type PreferencesSnapshot
} from '@/state/preferencesStore';
import { useShellStore } from '@/state/shellStore';

import { buildFavoriteFriendGroupOptions } from './settingsFavoriteGroupOptions';
import { resolveActiveSettingsTab } from './settingsOptions';
import { buildSettingsPageStateSections } from './settingsPageStateSections';
import { useSettingsActions } from './useSettingsActions';
import { useSettingsCommit } from './useSettingsCommit';
import { useSettingsEffects } from './useSettingsEffects';

const SETTINGS_PREFERENCE_KEYS = Object.keys(DEFAULT_PREFERENCES) as Array<
    keyof PreferencesSnapshot
>;

type SettingsSqliteTableSizes = Record<string, unknown>;
type SettingsTableLimitsDraft = {
    maxTableSize: string;
    searchLimit: string;
};

export function useSettingsPageState() {
    const locale = useShellStore((state) => state.locale);
    const zoomLevel = useShellStore((state) => state.zoomLevel);
    const lastSettingsTab = useShellStore((state) => state.lastSettingsTab);
    const setLastSettingsTab = useShellStore(
        (state) => state.setLastSettingsTab
    );
    const favoriteFriendGroups = useFavoriteStore(
        (state) => state.favoriteFriendGroups
    );
    const localFriendFavoriteGroups = useFavoriteStore(
        (state) => state.localFriendFavoriteGroups
    );
    const preferenceState = usePreferencesStore(
        useShallow((state) => {
            const snapshot: Record<string, unknown> = {};
            for (const key of SETTINGS_PREFERENCE_KEYS) {
                snapshot[key] = state[key];
            }
            return snapshot;
        })
    );
    const prefs = useMemo(
        () => normalizePreferenceSnapshot(preferenceState),
        [preferenceState]
    );
    const setPrefs = useCallback(
        (
            value:
                | PreferencesSnapshot
                | ((current: PreferencesSnapshot) => Record<string, unknown>)
        ) => {
            const store = usePreferencesStore.getState();
            const current = normalizePreferenceSnapshot(store);
            const next = typeof value === 'function' ? value(current) : value;
            store.patchPreferences(normalizePreferenceSnapshot(next));
        },
        []
    );
    const [sqliteTableSizes, setSqliteTableSizes] =
        useState<SettingsSqliteTableSizes>({});
    const [purgeDialogOpen, setPurgeDialogOpen] = useState(false);
    const [purgePeriod, setPurgePeriod] = useState('180');
    const [purgeInProgress, setPurgeInProgress] = useState(false);
    const [onlineVisitCount, setOnlineVisitCount] = useState<number | null>(
        null
    );
    const localFavoriteFriendsGroups = prefs.localFavoriteFriendsGroups;
    const setLocalFavoriteFriendsGroups = useCallback((groups: string[]) => {
        usePreferencesStore.getState().patchPreferences({
            localFavoriteFriendsGroups: groups
        });
    }, []);
    const [zoomInput, setZoomInput] = useState('100');
    const [searchParams, setSearchParams] = useSearchParams();
    const requestedTab = searchParams.get('tab') ?? '';
    const activeSettingsTab = resolveActiveSettingsTab(
        requestedTab,
        lastSettingsTab
    );

    useEffect(() => {
        if (
            requestedTab === activeSettingsTab &&
            requestedTab !== lastSettingsTab
        ) {
            setLastSettingsTab(requestedTab);
        }
    }, [activeSettingsTab, lastSettingsTab, requestedTab, setLastSettingsTab]);

    function setActiveSettingsTab(tab: string) {
        setLastSettingsTab(tab);
        setSearchParams(
            (current) => {
                current.set('tab', tab);
                return current;
            },
            { replace: true }
        );
    }
    const [webhookNotificationsDialogOpen, setWebhookNotificationsDialogOpen] =
        useState(false);
    const [tablePageSizesDialogOpen, setTablePageSizesDialogOpen] =
        useState(false);
    const [tableLimitsDialogOpen, setTableLimitsDialogOpen] = useState(false);
    const [tableLimitsDraft, setTableLimitsDraft] =
        useState<SettingsTableLimitsDraft>({
            maxTableSize: String(DEFAULT_MAX_TABLE_SIZE),
            searchLimit: String(DEFAULT_SEARCH_LIMIT)
        });
    const commit = useSettingsCommit();

    const {
        addFeedHiddenUser,
        savePreferenceValue,
        saveBoolPreference,
        saveStringPreference,
        saveFontFamilyPreference,
        selectCjkFontPack,
        saveTrustColor,
        resetTrustColors,
        refreshSqliteTableSizes,
        refreshOnlineVisits,
        openTablePageSizesDialog,
        openTableLimitsDialog,
        saveTableLimitsDialog,
        toggleLocalFavoriteFriendsGroup,
        resetUgcFolder,
        purgeAvatarFeedData,
        openUgcFolderSelector,
        handleCropInstancePrintsChange,
        handleFeedPersistenceDisabledChange,
        removeFeedHiddenUser,
        saveWebhookActivityFilters,
        setProxyEnabledPreference: saveProxyEnabledPreference,
        searchLimitError,
        tableLimitsSaveDisabled,
        tableMaxSizeError
    } = useSettingsActions({
        commit,
        localFavoriteFriendsGroups,
        prefs,
        purgePeriod,
        setLocalFavoriteFriendsGroups,
        setOnlineVisitCount,
        setPrefs,
        setPurgeDialogOpen,
        setPurgeInProgress,
        setSqliteTableSizes,
        setTableLimitsDialogOpen,
        setTableLimitsDraft,
        setTablePageSizesDialogOpen,
        tableLimitsDraft
    });
    useSettingsEffects({
        setZoomInput,
        zoomLevel
    });
    const {
        favoriteFriendGroupOptions,
        localFavoriteFriendGroupOptions,
        remoteFavoriteFriendGroupOptions,
        selectedFavoriteFriendGroupLabel
    } = useMemo(
        () =>
            buildFavoriteFriendGroupOptions({
                favoriteFriendGroups,
                localFriendFavoriteGroups,
                localFavoriteFriendsGroups
            }),
        [
            favoriteFriendGroups,
            localFavoriteFriendsGroups,
            localFriendFavoriteGroups
        ]
    );

    function normalizeRecentActionCooldownMinutes(value: string) {
        const parsed = Number.parseInt(value, 10);
        if (!Number.isFinite(parsed)) {
            return 60;
        }
        return Math.min(MINUTES_PER_DAY, Math.max(1, parsed));
    }

    async function saveInterfaceZoomLevel(value: string | number) {
        let savedZoom = zoomLevel;
        const saved = await commit(async () => {
            savedZoom = await setZoomLevelPreference(value);
        });
        if (saved) {
            setZoomInput(String(savedZoom));
        }
    }

    return buildSettingsPageStateSections({
        activeSettingsTab,
        addFeedHiddenUser,
        commit,
        handleCropInstancePrintsChange,
        handleFeedPersistenceDisabledChange,
        favoriteFriendGroupOptions,
        locale,
        localFavoriteFriendGroupOptions,
        localFavoriteFriendsGroups,
        normalizeRecentActionCooldownMinutes,
        onlineVisitCount,
        openTableLimitsDialog,
        openTablePageSizesDialog,
        openUgcFolderSelector,
        prefs,
        purgeAvatarFeedData,
        purgeDialogOpen,
        purgeInProgress,
        purgePeriod,
        refreshOnlineVisits,
        refreshSqliteTableSizes,
        remoteFavoriteFriendGroupOptions,
        removeFeedHiddenUser,
        resetTrustColors,
        resetUgcFolder,
        saveBoolPreference,
        saveFontFamilyPreference,
        saveInterfaceZoomLevel,
        savePreferenceValue,
        saveStringPreference,
        saveTableLimitsDialog,
        saveTrustColor,
        saveWebhookActivityFilters,
        searchLimitError,
        selectCjkFontPack,
        selectedFavoriteFriendGroupLabel,
        setAccessibleStatusIndicatorsPreference,
        setActiveSettingsTab,
        setAppLanguagePreference,
        setDataTableStripedPreference,
        setIntConfigPreference,
        setNotificationLayoutPreference,
        setPrefs,
        setPurgeDialogOpen,
        setProxyEnabledPreference: saveProxyEnabledPreference,
        setPurgePeriod,
        setRecentActionCooldownEnabledPreference,
        setRecentActionCooldownMinutesPreference,
        setSaveInstanceEmojiPreference,
        setSaveInstancePrintsPreference,
        setSaveInstanceStickersPreference,
        setShowNewDashboardButtonPreference,
        setTableDensityPreference,
        setTableLimitsDialogOpen,
        setTableLimitsDraft,
        setTablePageSizesDialogOpen,
        setWebhookNotificationsDialogOpen,
        setZoomInput,
        setZoomLevelPreference,
        sqliteTableSizes,
        tableLimitsDialogOpen,
        tableLimitsDraft,
        tableLimitsSaveDisabled,
        tableMaxSizeError,
        tablePageSizesDialogOpen,
        toggleLocalFavoriteFriendsGroup,
        webhookNotificationsDialogOpen,
        zoomInput,
        zoomLevel
    });
}
