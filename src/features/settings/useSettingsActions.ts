import { useTranslation } from 'react-i18next';

import { commands } from '@/platform/tauri/bindings';
import configRepository from '@/repositories/configRepository';
import vrchatAuthRepository from '@/repositories/vrchatAuthRepository';
import {
    addFeedHiddenUserPreference,
    setBoolConfigPreference,
    setFeedPersistenceDisabledPreference,
    setCropInstancePrintsPreference,
    setIntConfigPreference,
    setLocalFavoriteFriendsGroupsPreference,
    setProxyEnabledPreference,
    setStringConfigPreference,
    setTableLimitsPreference,
    setTrustColorPreference,
    setUserGeneratedContentPathPreference,
    setWebhookActivityFiltersPreference,
    loadTrustColorPreference,
    removeFeedHiddenUserPreference,
    resetTrustColorsPreference
} from '@/services/preferencesService';
import {
    applyAppFontPreferences,
    normalizeAppCjkFontPack,
    normalizeAppFontFamily
} from '@/services/themeService';
import { toast } from '@/services/toastService';
import { APP_FONT_DEFAULT_KEY } from '@/shared/constants/fonts';
import {
    DEFAULT_MAX_TABLE_SIZE,
    DEFAULT_SEARCH_LIMIT,
    SEARCH_LIMIT_MAX,
    SEARCH_LIMIT_MIN,
    TABLE_MAX_SIZE_MAX,
    TABLE_MAX_SIZE_MIN
} from '@/shared/constants/settings';
import { useModalStore } from '@/state/modalStore';
import {
    normalizePreferenceSnapshot,
    usePreferencesStore
} from '@/state/preferencesStore';
import { useRuntimeStore } from '@/state/runtimeStore';

import type { createDefaultSettingsPrefs } from './settingsDefaultPrefs';
import { parseIntegerInput } from './settingsValues';
import { createSettingsMaintenanceActions } from './useSettingsMaintenanceActions';
import { useSettingsPreferenceActions } from './useSettingsPreferenceActions';

type SettingsPreferenceActionDeps = Parameters<
    typeof useSettingsPreferenceActions
>[0];
type SettingsMaintenanceActionDeps = Parameters<
    typeof createSettingsMaintenanceActions
>[0];
type SettingsPagePrefsDraft = ReturnType<typeof createDefaultSettingsPrefs>;
type SettingsPagePrefsSetter = (
    value:
        | SettingsPagePrefsDraft
        | ((current: SettingsPagePrefsDraft) => SettingsPagePrefsDraft)
) => void;
type SettingsActionsDeps = Pick<
    SettingsPreferenceActionDeps,
    | 'commit'
    | 'localFavoriteFriendsGroups'
    | 'prefs'
    | 'setLocalFavoriteFriendsGroups'
    | 'setOnlineVisitCount'
    | 'setSqliteTableSizes'
    | 'setTableLimitsDialogOpen'
    | 'setTableLimitsDraft'
    | 'setTablePageSizesDialogOpen'
    | 'tableLimitsDraft'
> &
    Pick<
        SettingsMaintenanceActionDeps,
        'purgePeriod' | 'setPurgeDialogOpen' | 'setPurgeInProgress'
    > & { setPrefs: SettingsPagePrefsSetter };

export function useSettingsActions(deps: SettingsActionsDeps) {
    const { t } = useTranslation();
    const confirm = useModalStore((state) => state.confirm);
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentUserEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    const auth = {
        currentUserId,
        currentUserEndpoint
    };
    const tableMaxSizeValue = Number.parseInt(
        deps.tableLimitsDraft.maxTableSize,
        10
    );
    const tableMaxSizeError =
        !Number.isFinite(tableMaxSizeValue) ||
        tableMaxSizeValue < TABLE_MAX_SIZE_MIN ||
        tableMaxSizeValue > TABLE_MAX_SIZE_MAX
            ? t('prompt.table_entries_settings.table_max_entries_error', {
                  min: TABLE_MAX_SIZE_MIN,
                  max: TABLE_MAX_SIZE_MAX
              })
            : '';
    const searchLimitValue = Number.parseInt(
        deps.tableLimitsDraft.searchLimit,
        10
    );
    const searchLimitError =
        !Number.isFinite(searchLimitValue) ||
        searchLimitValue < SEARCH_LIMIT_MIN ||
        searchLimitValue > SEARCH_LIMIT_MAX
            ? t('prompt.table_entries_settings.search_limit_returns_error', {
                  min: SEARCH_LIMIT_MIN,
                  max: SEARCH_LIMIT_MAX
              })
            : '';
    const tableLimitsSaveDisabled = Boolean(
        tableMaxSizeError || searchLimitError
    );
    const actionDeps = {
        ...deps,
        APP_FONT_DEFAULT_KEY,
        DEFAULT_MAX_TABLE_SIZE,
        DEFAULT_SEARCH_LIMIT,
        applyAppFontPreferences,
        auth,
        cleanupAvatarFeedHistory: commands.appAvatarFeedHistoryCleanup,
        configRepository,
        confirm,
        cropAllPrints: commands.appCropAllPrints,
        getUgcPhotoLocation: commands.appGetUgcPhotoLocation,
        normalizeAppCjkFontPack,
        normalizeAppFontFamily,
        normalizePreferenceSnapshot,
        parseIntegerInput,
        loadTrustColorPreference,
        resetTrustColorsPreference,
        setBoolConfigPreference,
        setFeedPersistenceDisabledPreference,
        setCropInstancePrintsPreference,
        setIntConfigPreference,
        setLocalFavoriteFriendsGroupsPreference,
        setProxyEnabledPreference,
        setStringConfigPreference,
        setTableLimitsPreference,
        setTrustColorPreference,
        setUserGeneratedContentPathPreference,
        setWebhookActivityFiltersPreference,
        t,
        tableLimitsSaveDisabled,
        toast,
        usePreferencesStore,
        setPrefs: deps.setPrefs,
        vrchatAuthRepository
    };
    const preferenceActions = useSettingsPreferenceActions(actionDeps);
    const maintenanceActions = createSettingsMaintenanceActions({
        ...actionDeps,
        ...preferenceActions
    });
    function normalizeCurrentFeedHiddenUsers() {
        return normalizePreferenceSnapshot({
            feedHiddenUsers: deps.prefs.feedHiddenUsers
        }).feedHiddenUsers;
    }
    async function addFeedHiddenUser(userId: string) {
        const previous = normalizeCurrentFeedHiddenUsers();
        const next = normalizePreferenceSnapshot({
            feedHiddenUsers: [...previous, userId]
        }).feedHiddenUsers;
        await deps.commit(
            () => addFeedHiddenUserPreference(userId),
            () => {
                deps.setPrefs((current) => ({
                    ...current,
                    feedHiddenUsers: next
                }));
                return () =>
                    deps.setPrefs((current) => ({
                        ...current,
                        feedHiddenUsers: previous
                    }));
            }
        );
    }
    async function removeFeedHiddenUser(userId: string) {
        const normalizedUserId = userId.trim();
        if (!normalizedUserId) {
            return;
        }
        const previous = normalizeCurrentFeedHiddenUsers();
        const next = previous.filter((id) => id !== normalizedUserId);
        await deps.commit(
            () => removeFeedHiddenUserPreference(normalizedUserId),
            () => {
                deps.setPrefs((current) => ({
                    ...current,
                    feedHiddenUsers: next
                }));
                return () =>
                    deps.setPrefs((current) => ({
                        ...current,
                        feedHiddenUsers: previous
                    }));
            }
        );
    }
    return {
        ...preferenceActions,
        ...maintenanceActions,
        addFeedHiddenUser,
        removeFeedHiddenUser,
        searchLimitError,
        tableLimitsSaveDisabled,
        tableMaxSizeError
    };
}
