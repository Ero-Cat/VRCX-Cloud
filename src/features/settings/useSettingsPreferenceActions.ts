import { commands } from '@/platform/tauri/bindings';
import type {
    BoolConfigPreferenceKey,
    StringConfigPreferenceKey
} from '@/services/preferencesService';
import type { AppToastOptions } from '@/services/toastService';
import type { OverlayActivityTypeDefinition } from '@/shared/constants/overlayActivityFilters';
import type {
    PreferencesSnapshot,
    PreferencesStoreState,
    TrustColorKey
} from '@/state/preferencesStore';

type PreferenceKey = Extract<keyof PreferencesSnapshot, string>;
type NormalizedConfigKey<Key extends string> = Key extends `VRCX_${infer Name}`
    ? Name
    : Key;
type BoolPreferenceKey = NormalizedConfigKey<BoolConfigPreferenceKey> &
    PreferenceKey;
type StringPreferenceKey = NormalizedConfigKey<StringConfigPreferenceKey> &
    PreferenceKey;
type PreferenceAction = () => void;
type PreferenceRollback = void | (() => void);
export type SettingsActionPrefs = PreferencesSnapshot;
type SettingsPrefs = SettingsActionPrefs;
type StateSetter<Value> = (value: Value | ((current: Value) => Value)) => void;
type SettingsPreferenceActionsDeps = {
    APP_FONT_DEFAULT_KEY: string;
    DEFAULT_MAX_TABLE_SIZE: number;
    DEFAULT_SEARCH_LIMIT: number;
    applyAppFontPreferences: (preferences: {
        fontFamily: string;
        customFontFamily: string;
        cjkFontPack: string;
    }) => void;
    auth: {
        currentUserEndpoint?: string | null;
        currentUserId?: string | null;
    };
    commit: (
        action: PreferenceAction,
        optimistic?: () => PreferenceRollback
    ) => Promise<boolean>;
    configRepository: {
        setMany(entries: Array<[string, string]>): Promise<void>;
    };
    loadTrustColorPreference: () => Promise<PreferencesSnapshot['trustColor']>;
    localFavoriteFriendsGroups: string[];
    normalizeAppCjkFontPack: (value: string) => string;
    normalizeAppFontFamily: (value: string) => string;
    parseIntegerInput: (value: string | number, fallback: number) => number;
    prefs: SettingsPrefs;
    resetTrustColorsPreference: () => Promise<
        PreferencesSnapshot['trustColor']
    >;
    setBoolConfigPreference: (
        key: BoolConfigPreferenceKey,
        value: boolean
    ) => Promise<void>;
    setLocalFavoriteFriendsGroups: (value: string[]) => void;
    setLocalFavoriteFriendsGroupsPreference: (
        value: string[]
    ) => Promise<string[]>;
    setOnlineVisitCount: (value: number) => void;
    setPrefs: StateSetter<SettingsPrefs>;
    setProxyEnabledPreference: (value: boolean) => Promise<boolean>;
    setSqliteTableSizes: (value: Record<string, unknown>) => void;
    setStringConfigPreference: (
        key: StringConfigPreferenceKey,
        value: string
    ) => Promise<void>;
    setTableLimitsDialogOpen: (value: boolean) => void;
    setTableLimitsDraft: (value: {
        maxTableSize: string;
        searchLimit: string;
    }) => void;
    setTableLimitsPreference: (value: {
        maxTableSize: number;
        searchLimit: number;
    }) => Promise<PreferencesSnapshot['tableLimits']>;
    setTablePageSizesDialogOpen: (value: boolean) => void;
    setTrustColorPreference: (
        key: TrustColorKey,
        value: string
    ) => Promise<PreferencesSnapshot['trustColor']>;
    setWebhookActivityFiltersPreference: (
        value: PreferencesSnapshot['webhookActivityFilters']
    ) => Promise<PreferencesSnapshot['webhookActivityFilters']>;
    t: (key: string) => string;
    tableLimitsDraft: {
        maxTableSize: string;
        searchLimit: string;
    };
    tableLimitsSaveDisabled: boolean;
    toast: {
        add(options: AppToastOptions): void;
    };
    usePreferencesStore: {
        getState(): Pick<PreferencesStoreState, 'tableLimits'>;
    };
    vrchatAuthRepository: {
        getOnlineVisits(): Promise<{ json: unknown }>;
    };
};

type FontPreferencesInput = Partial<{
    cjkFontPack: string;
    customFontFamily: string;
    fontFamily: string;
}>;

export function useSettingsPreferenceActions({
    APP_FONT_DEFAULT_KEY,
    DEFAULT_MAX_TABLE_SIZE,
    DEFAULT_SEARCH_LIMIT,
    applyAppFontPreferences,
    auth,
    commit,
    configRepository,
    loadTrustColorPreference,
    localFavoriteFriendsGroups,
    normalizeAppCjkFontPack,
    normalizeAppFontFamily,
    parseIntegerInput,
    prefs,
    resetTrustColorsPreference,
    setBoolConfigPreference,
    setLocalFavoriteFriendsGroups,
    setLocalFavoriteFriendsGroupsPreference,
    setOnlineVisitCount,
    setPrefs,
    setProxyEnabledPreference,
    setSqliteTableSizes,
    setStringConfigPreference,
    setTableLimitsDialogOpen,
    setTableLimitsDraft,
    setTableLimitsPreference,
    setTablePageSizesDialogOpen,
    setTrustColorPreference,
    setWebhookActivityFiltersPreference,
    t,
    tableLimitsDraft,
    tableLimitsSaveDisabled,
    toast,
    usePreferencesStore,
    vrchatAuthRepository
}: SettingsPreferenceActionsDeps) {
    async function savePreferenceValue<K extends PreferenceKey>(
        key: K,
        value: PreferencesSnapshot[K],
        action: PreferenceAction
    ) {
        return commit(action, () => {
            const previous = prefs[key];
            setPrefs((current) => ({
                ...current,
                [key]: value
            }));
            return () =>
                setPrefs((current) => ({
                    ...current,
                    [key]: previous
                }));
        });
    }
    async function saveBoolPreference(
        key: BoolPreferenceKey,
        configKey: BoolConfigPreferenceKey,
        value: boolean
    ) {
        const enabled = value === true;
        await savePreferenceValue(key, enabled, () =>
            setBoolConfigPreference(configKey, enabled)
        );
    }
    async function saveStringPreference(
        key: StringPreferenceKey,
        configKey: StringConfigPreferenceKey,
        value: string
    ) {
        await savePreferenceValue(key, value, () =>
            setStringConfigPreference(configKey, value)
        );
    }
    async function saveFontPreferences({
        fontFamily = prefs.appFontFamily,
        cjkFontPack = prefs.appCjkFontPack,
        customFontFamily = prefs.customFontFamily
    }: FontPreferencesInput = {}) {
        const nextFontFamily = normalizeAppFontFamily(fontFamily);
        const nextCjkFontPack = normalizeAppCjkFontPack(cjkFontPack);
        await configRepository.setMany([
            ['VRCX_fontFamily', nextFontFamily],
            ['VRCX_cjkFontPack', nextCjkFontPack]
        ]);
        setPrefs((current) => ({
            ...current,
            appFontFamily: nextFontFamily,
            appCjkFontPack: nextCjkFontPack
        }));
        applyAppFontPreferences({
            fontFamily: nextFontFamily,
            customFontFamily,
            cjkFontPack: nextCjkFontPack
        });
    }
    async function saveFontFamilyPreference(
        fontFamily: string,
        customFontFamily: string = prefs.customFontFamily
    ) {
        await saveFontPreferences({
            fontFamily,
            customFontFamily
        });
    }
    async function selectCjkFontPack(cjkFontPack: string) {
        await saveFontPreferences({
            fontFamily:
                prefs.appFontFamily === 'custom'
                    ? APP_FONT_DEFAULT_KEY
                    : prefs.appFontFamily,
            cjkFontPack
        });
    }
    async function restorePersistedTrustColors() {
        const persisted = await loadTrustColorPreference();
        setPrefs((current) => ({
            ...current,
            trustColor: persisted
        }));
    }
    async function saveTrustColor(key: TrustColorKey, value: string) {
        try {
            const nextTrustColor = await setTrustColorPreference(key, value);
            setPrefs((current) => ({
                ...current,
                trustColor: nextTrustColor
            }));
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('view.settings.toast.failed_to_save_trust_color')
            });
            await restorePersistedTrustColors();
        }
    }
    async function resetTrustColors() {
        try {
            const nextTrustColor = await resetTrustColorsPreference();
            setPrefs((current) => ({
                ...current,
                trustColor: nextTrustColor
            }));
            toast.add({ type: 'success', title: t('common.settings_saved') });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('view.settings.toast.failed_to_save_trust_color')
            });
        }
    }
    async function refreshSqliteTableSizes() {
        try {
            const sizes = await commands.appDatabaseMaintenanceTableSizesGet(
                auth.currentUserId || ''
            );
            setSqliteTableSizes({
                gps: sizes.gps,
                status: sizes.status,
                bio: sizes.bio,
                avatar: sizes.avatar,
                onlineOffline: sizes.onlineOffline,
                friendLogHistory: sizes.friendLogHistory,
                notification: sizes.notification,
                location: sizes.location,
                joinLeave: sizes.joinLeave,
                portalSpawn: sizes.portalSpawn,
                videoPlay: sizes.videoPlay,
                event: sizes.event,
                external: sizes.external
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'view.settings.toast.failed_to_refresh_sqlite_table_sizes'
                          )
            });
        }
    }
    async function refreshOnlineVisits() {
        try {
            const response = await vrchatAuthRepository.getOnlineVisits();
            setOnlineVisitCount(Number(response.json) || 0);
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'view.settings.toast.failed_to_refresh_online_user_count'
                          )
            });
        }
    }
    async function openTablePageSizesDialog() {
        setTablePageSizesDialogOpen(true);
    }
    async function openTableLimitsDialog() {
        const { maxTableSize, searchLimit } =
            usePreferencesStore.getState().tableLimits;
        setTableLimitsDraft({
            maxTableSize: String(
                parseIntegerInput(maxTableSize, DEFAULT_MAX_TABLE_SIZE)
            ),
            searchLimit: String(
                parseIntegerInput(searchLimit, DEFAULT_SEARCH_LIMIT)
            )
        });
        setTableLimitsDialogOpen(true);
    }
    async function saveTableLimitsDialog() {
        if (tableLimitsSaveDisabled) {
            return;
        }
        const nextMaxTableSize = Number.parseInt(
            tableLimitsDraft.maxTableSize,
            10
        );
        const nextSearchLimit = Number.parseInt(
            tableLimitsDraft.searchLimit,
            10
        );
        let savedLimits = prefs.tableLimits;
        const saved = await commit(async () => {
            savedLimits = await setTableLimitsPreference({
                maxTableSize: nextMaxTableSize,
                searchLimit: nextSearchLimit
            });
        });
        if (!saved) {
            return;
        }
        setPrefs((current) => ({
            ...current,
            tableLimits: savedLimits
        }));
        setTableLimitsDialogOpen(false);
        toast.add({ type: 'success', title: t('common.settings_saved') });
    }
    async function toggleLocalFavoriteFriendsGroup(
        groupKey: string,
        checked: boolean
    ) {
        const previousGroups = localFavoriteFriendsGroups;
        const nextGroups = checked
            ? Array.from(new Set([...localFavoriteFriendsGroups, groupKey]))
            : localFavoriteFriendsGroups.filter((value) => value !== groupKey);
        await commit(
            () => setLocalFavoriteFriendsGroupsPreference(nextGroups),
            () => {
                setLocalFavoriteFriendsGroups(nextGroups);
                return () => {
                    setLocalFavoriteFriendsGroups(previousGroups);
                };
            }
        );
    }
    async function saveWebhookActivityFilters(
        value: PreferencesSnapshot['webhookActivityFilters'],
        definitions?: OverlayActivityTypeDefinition[]
    ) {
        void definitions;
        let savedFilters = prefs.webhookActivityFilters;
        const previousFilters = prefs.webhookActivityFilters;
        const saved = await commit(
            async () => {
                savedFilters = await setWebhookActivityFiltersPreference(value);
            },
            () => {
                setPrefs((current) => ({
                    ...current,
                    webhookActivityFilters: value
                }));
                return () =>
                    setPrefs((current) => ({
                        ...current,
                        webhookActivityFilters: previousFilters
                    }));
            }
        );
        if (!saved) {
            return null;
        }
        setPrefs((current) => ({
            ...current,
            webhookActivityFilters: savedFilters
        }));
        toast.add({ type: 'success', title: t('common.settings_saved') });
        return savedFilters;
    }
    return {
        commit,
        savePreferenceValue,
        saveBoolPreference,
        saveStringPreference,
        saveFontFamilyPreference,
        selectCjkFontPack,
        saveTrustColor,
        resetTrustColors,
        refreshSqliteTableSizes,
        refreshOnlineVisits,
        setProxyEnabledPreference,
        openTablePageSizesDialog,
        openTableLimitsDialog,
        saveTableLimitsDialog,
        toggleLocalFavoriteFriendsGroup,
        saveWebhookActivityFilters
    };
}
