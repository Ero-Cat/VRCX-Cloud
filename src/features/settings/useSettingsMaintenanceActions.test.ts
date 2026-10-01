import { describe, expect, it, vi } from 'vitest';

import type { AppToastOptions } from '@/services/toastService';

import { createDefaultSettingsPrefs } from './settingsDefaultPrefs';
import { DEFAULT_WEBHOOK_ACTIVITY_FILTERS } from './settingsValues';
import { createSettingsMaintenanceActions } from './useSettingsMaintenanceActions';

function createMaintenanceActions({
    cleanupAvatarFeedHistory = async () => ({
        deletedRows: 0,
        status: 'completed' as const,
        optimizationError: null
    }),
    confirm,
    isGameRunning = false,
    setFeedPersistenceDisabledPreference = async () => undefined,
    setPurgeDialogOpen = () => undefined,
    toastWarning = () => undefined,
    toastError = () => undefined
}: {
    cleanupAvatarFeedHistory?: () => Promise<{
        deletedRows: number;
        status: 'completed' | 'optimizationFailed';
        optimizationError: string | null;
    }>;
    confirm: (options: {
        title: string;
        description: string;
    }) => Promise<{ ok: boolean }>;
    isGameRunning?: boolean;
    setFeedPersistenceDisabledPreference?: (disabled: boolean) => Promise<void>;
    setPurgeDialogOpen?: (open: boolean) => void;
    toastWarning?: (options: AppToastOptions) => void;
    toastError?: (options: AppToastOptions) => void;
}) {
    void isGameRunning;
    const prefs = createDefaultSettingsPrefs();
    return createSettingsMaintenanceActions({
        cleanupAvatarFeedHistory,
        commit: async () => true,
        confirm,
        cropAllPrints: async () => null,
        getUgcPhotoLocation: async () => '',
        prefs: {
            ...prefs,
            webhookActivityFilters: DEFAULT_WEBHOOK_ACTIVITY_FILTERS
        },
        purgePeriod: '180',
        savePreferenceValue: async (_key, _value, action) => {
            await action();
            return true;
        },
        setCropInstancePrintsPreference: async () => undefined,
        setFeedPersistenceDisabledPreference,
        setPrefs: () => undefined,
        setPurgeDialogOpen,
        setPurgeInProgress: () => undefined,
        setUserGeneratedContentPathPreference: async () => '',
        t: (key) => key,
        toast: {
            add: (options: AppToastOptions) => {
                switch (options.type) {
                    case 'warning':
                        return toastWarning(options);
                    case 'error':
                        return toastError(options);
                    default:
                        throw new Error(
                            'Unhandled toast type: ' + options.type
                        );
                }
            }
        }
    });
}

describe('handleFeedPersistenceDisabledChange', () => {
    it('keeps Feed history enabled when disabling is not confirmed', async () => {
        const setFeedPersistenceDisabledPreference = vi.fn(
            async () => undefined
        );
        const actions = createMaintenanceActions({
            confirm: async () => ({ ok: false }),
            setFeedPersistenceDisabledPreference
        });

        await actions.handleFeedPersistenceDisabledChange(true);

        expect(setFeedPersistenceDisabledPreference).not.toHaveBeenCalled();
    });

    it('can switch Feed persistence while VRChat is running', async () => {
        const setFeedPersistenceDisabledPreference = vi.fn(
            async () => undefined
        );
        const actions = createMaintenanceActions({
            confirm: async () => ({ ok: true }),
            isGameRunning: true,
            setFeedPersistenceDisabledPreference
        });

        await actions.handleFeedPersistenceDisabledChange(true);

        expect(setFeedPersistenceDisabledPreference).toHaveBeenCalledWith(true);
    });
});

describe('purgeAvatarFeedData', () => {
    it('reports a completed purge separately from a failed optimization', async () => {
        const setPurgeDialogOpen = vi.fn();
        const toastWarning = vi.fn();
        const actions = createMaintenanceActions({
            cleanupAvatarFeedHistory: async () => ({
                deletedRows: 12,
                status: 'optimizationFailed',
                optimizationError: 'vacuum failed'
            }),
            confirm: async () => ({ ok: false }),
            setPurgeDialogOpen,
            toastWarning
        });

        await actions.purgeAvatarFeedData();

        expect(setPurgeDialogOpen).toHaveBeenCalledWith(false);
        expect(toastWarning).toHaveBeenCalledWith(
            expect.objectContaining({
                type: 'warning',
                title: 'view.settings.advanced.advanced.database_cleanup.purge_optimization_failed'
            })
        );
    });
});
