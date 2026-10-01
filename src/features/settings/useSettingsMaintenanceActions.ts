import type { commands } from '@/platform/tauri/bindings';
import {
    openFolderSelectorDialog,
    restartApplication
} from '@/services/shellIntegrationService';
import type { AppToastOptions } from '@/services/toastService';
import { useRuntimeStore } from '@/state/runtimeStore';

import type {
    SettingsActionPrefs,
    useSettingsPreferenceActions
} from './useSettingsPreferenceActions';

type PreferenceAction = () => void;
type PreferenceRollback = void | (() => void);
type PreferenceActions = ReturnType<typeof useSettingsPreferenceActions>;
type SettingsPrefs = SettingsActionPrefs;
type StateSetter<Value> = {
    bivarianceHack(
        value: Value | ((current: Value) => Value | Record<string, unknown>)
    ): void;
}['bivarianceHack'];
type SettingsDialogResult = {
    ok: boolean;
    reason?: string;
    value?: string;
};
type SettingsConfirmOptions = {
    title: string;
    description: string;
    confirmText?: string;
    alternativeText?: string;
    cancelText?: string;
    dismissible?: boolean;
    destructive?: boolean;
};
type SettingsToast = {
    add(options: AppToastOptions): void;
};
type SettingsMaintenanceActionsDeps = {
    cleanupAvatarFeedHistory: typeof commands.appAvatarFeedHistoryCleanup;
    commit: (
        action: PreferenceAction,
        optimistic?: () => PreferenceRollback
    ) => Promise<boolean>;
    confirm: (options: SettingsConfirmOptions) => Promise<SettingsDialogResult>;
    cropAllPrints: typeof commands.appCropAllPrints;
    getUgcPhotoLocation: typeof commands.appGetUgcPhotoLocation;
    prefs: SettingsPrefs;
    purgePeriod: string;
    savePreferenceValue: PreferenceActions['savePreferenceValue'];
    setCropInstancePrintsPreference: (value: boolean) => Promise<void>;
    setFeedPersistenceDisabledPreference: (disabled: boolean) => Promise<void>;
    setPrefs: StateSetter<SettingsPrefs>;
    setPurgeDialogOpen: (value: boolean) => void;
    setPurgeInProgress: (value: boolean) => void;
    setUserGeneratedContentPathPreference: (value: string) => Promise<string>;
    t: (key: string, options?: Record<string, unknown>) => string;
    toast: SettingsToast;
};

export function createSettingsMaintenanceActions({
    cleanupAvatarFeedHistory,
    commit,
    confirm,
    cropAllPrints,
    getUgcPhotoLocation,
    prefs,
    purgePeriod,
    savePreferenceValue,
    setCropInstancePrintsPreference,
    setFeedPersistenceDisabledPreference,
    setPrefs,
    setPurgeDialogOpen,
    setPurgeInProgress,
    setUserGeneratedContentPathPreference,
    t,
    toast
}: SettingsMaintenanceActionsDeps) {
    async function resetUgcFolder() {
        await commit(
            () => setUserGeneratedContentPathPreference(''),
            () => {
                const previous = prefs.userGeneratedContentPath;
                setPrefs((current) => ({
                    ...current,
                    userGeneratedContentPath: ''
                }));
                return () =>
                    setPrefs((current) => ({
                        ...current,
                        userGeneratedContentPath: previous
                    }));
            }
        );
    }
    async function purgeAvatarFeedData() {
        const cutoffDate =
            purgePeriod === 'all'
                ? null
                : (() => {
                      const cutoff = new Date();
                      cutoff.setDate(
                          cutoff.getDate() - Number.parseInt(purgePeriod, 10)
                      );
                      return cutoff.toJSON();
                  })();
        setPurgeInProgress(true);
        useRuntimeStore.getState().setDatabaseMaintenanceActive(true);
        try {
            const outcome = await cleanupAvatarFeedHistory(cutoffDate);
            setPurgeDialogOpen(false);
            if (outcome.status === 'optimizationFailed') {
                toast.add({
                    type: 'warning',
                    title: t(
                        'view.settings.advanced.advanced.database_cleanup.purge_optimization_failed',
                        { error: outcome.optimizationError ?? '' }
                    )
                });
                return;
            }
            toast.add({
                type: 'success',
                title: t(
                    'view.settings.advanced.advanced.database_cleanup.purge_complete'
                )
            });
            await new Promise<void>((resolve) =>
                window.setTimeout(resolve, 1500)
            );
            await restartApplication();
        } catch (error) {
            toast.add({
                type: 'error',
                title: t(
                    'view.settings.advanced.advanced.database_cleanup.purge_failed',
                    {
                        error:
                            error instanceof Error
                                ? error.message
                                : String(error)
                    }
                )
            });
        } finally {
            useRuntimeStore.getState().setDatabaseMaintenanceActive(false);
            setPurgeInProgress(false);
        }
    }
    async function openUgcFolderSelector() {
        const selectedPath = await openFolderSelectorDialog(
            prefs.userGeneratedContentPath || ''
        ).catch((error: unknown) => {
            toast.add({
                type: 'error',
                title: error instanceof Error ? error.message : String(error)
            });
            return '';
        });
        if (!selectedPath) {
            return;
        }
        await savePreferenceValue(
            'userGeneratedContentPath',
            selectedPath,
            () => setUserGeneratedContentPathPreference(selectedPath)
        );
    }
    async function promptCropExistingPrints() {
        const result = await confirm({
            title: t('view.settings.modal.crop_existing_prints'),
            description: t(
                'view.settings.modal.crop_already_saved_instance_prints_in_the_config'
            ),
            confirmText: t('view.settings.modal.crop_prints'),
            cancelText: t('view.settings.modal.skip')
        });
        if (!result.ok) {
            return;
        }
        const ugcFolderPath = await getUgcPhotoLocation(
            prefs.userGeneratedContentPath
        );
        await cropAllPrints(ugcFolderPath);
        toast.add({
            type: 'success',
            title: t('view.settings.label.existing_saved_prints_cropped')
        });
    }
    async function handleCropInstancePrintsChange(enabled: boolean) {
        const saved = await commit(
            () => setCropInstancePrintsPreference(enabled),
            () => {
                setPrefs((current) => ({
                    ...current,
                    cropInstancePrints: enabled
                }));
                return () =>
                    setPrefs((current) => ({
                        ...current,
                        cropInstancePrints: !enabled
                    }));
            }
        );
        if (saved && enabled) {
            await promptCropExistingPrints().catch((error: unknown) => {
                toast.add({
                    type: 'error',
                    title:
                        error instanceof Error
                            ? error.message
                            : t(
                                  'view.settings.toast.failed_to_crop_existing_prints'
                              )
                });
            });
        }
    }
    async function handleFeedPersistenceDisabledChange(disabled: boolean) {
        if (disabled) {
            const result = await confirm({
                title: t('confirm.title'),
                description: t('confirm.disable_feed_persistence')
            });
            if (!result.ok) {
                return;
            }
        }
        await savePreferenceValue('feedPersistenceDisabled', disabled, () =>
            setFeedPersistenceDisabledPreference(disabled)
        );
    }
    return {
        resetUgcFolder,
        purgeAvatarFeedData,
        openUgcFolderSelector,
        handleCropInstancePrintsChange,
        handleFeedPersistenceDisabledChange
    };
}
