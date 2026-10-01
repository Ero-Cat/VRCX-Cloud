import type { SettingsSectionInput } from '../settingsPageStateSectionTypes';

type DialogsSectionInput = SettingsSectionInput<
    | 'tablePageSizesDialogOpen'
    | 'setTablePageSizesDialogOpen'
    | 'setPrefs'
    | 'tableLimitsDialogOpen'
    | 'setTableLimitsDialogOpen'
    | 'tableLimitsDraft'
    | 'setTableLimitsDraft'
    | 'tableMaxSizeError'
    | 'searchLimitError'
    | 'tableLimitsSaveDisabled'
    | 'saveTableLimitsDialog'
    | 'purgeDialogOpen'
    | 'setPurgeDialogOpen'
    | 'purgePeriod'
    | 'setPurgePeriod'
    | 'purgeInProgress'
    | 'purgeAvatarFeedData'
    | 'webhookNotificationsDialogOpen'
    | 'setWebhookNotificationsDialogOpen'
    | 'prefs'
    | 'saveWebhookActivityFilters'
>;

export function buildDialogsSection({
    tablePageSizesDialogOpen,
    setTablePageSizesDialogOpen,
    setPrefs,
    tableLimitsDialogOpen,
    setTableLimitsDialogOpen,
    tableLimitsDraft,
    setTableLimitsDraft,
    tableMaxSizeError,
    searchLimitError,
    tableLimitsSaveDisabled,
    saveTableLimitsDialog,
    purgeDialogOpen,
    setPurgeDialogOpen,
    purgePeriod,
    setPurgePeriod,
    purgeInProgress,
    purgeAvatarFeedData,
    webhookNotificationsDialogOpen,
    setWebhookNotificationsDialogOpen,
    prefs,
    saveWebhookActivityFilters
}: DialogsSectionInput) {
    return {
        tablePageSizesDialogOpen,
        setTablePageSizesDialogOpen,
        setPrefs,
        tableLimitsDialogOpen,
        setTableLimitsDialogOpen,
        tableLimitsDraft,
        setTableLimitsDraft,
        tableMaxSizeError,
        searchLimitError,
        tableLimitsSaveDisabled,
        saveTableLimitsDialog,
        purgeDialogOpen,
        setPurgeDialogOpen,
        purgePeriod,
        setPurgePeriod,
        purgeInProgress,
        purgeAvatarFeedData,
        webhookNotificationsDialogOpen,
        setWebhookNotificationsDialogOpen,
        webhookActivityFilters: prefs.webhookActivityFilters,
        saveWebhookActivityFilters
    };
}
