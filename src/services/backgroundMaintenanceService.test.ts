import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    isHostCapabilityAvailable: vi.fn(),
    runRuntimeTelemetryJob: vi.fn(),
    appRegistryBackupMaintenanceRun: vi.fn()
}));

vi.mock('@/platform/native/bindings', () => ({
    commands: {
        appRegistryBackupMaintenanceRun: mocks.appRegistryBackupMaintenanceRun
    }
}));

vi.mock('./hostCapabilityService', () => ({
    isHostCapabilityAvailable: mocks.isHostCapabilityAvailable
}));

vi.mock('./runtimeJobTelemetryService', () => ({
    runRuntimeTelemetryJob: mocks.runRuntimeTelemetryJob
}));

vi.mock('./i18nService', () => ({
    default: {
        t: (key: string, values?: Record<string, unknown>) =>
            values ? `${key}:${JSON.stringify(values)}` : key
    }
}));

import {
    runForegroundUpdateRegistryBackupMaintenance,
    runStartupMaintenance
} from './backgroundMaintenanceService';

describe('backgroundMaintenanceService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mocks.isHostCapabilityAvailable.mockReturnValue(false);
        mocks.runRuntimeTelemetryJob.mockImplementation(
            async (_metadata: unknown, task: () => Promise<unknown>) => task()
        );
    });

    it('runStartupMaintenance only runs registry backup maintenance', async () => {
        mocks.isHostCapabilityAvailable.mockReturnValue(true);
        mocks.appRegistryBackupMaintenanceRun.mockResolvedValue({
            restorePromptNeeded: false
        });

        await runStartupMaintenance();

        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenCalledWith(
            'foreground-startup'
        );
    });

    it('runForegroundUpdateRegistryBackupMaintenance runs registry backup maintenance independently of the update check', async () => {
        mocks.isHostCapabilityAvailable.mockReturnValue(true);
        mocks.appRegistryBackupMaintenanceRun.mockResolvedValue({
            restorePromptNeeded: false
        });

        await runForegroundUpdateRegistryBackupMaintenance();

        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenCalledWith(
            'foreground-update'
        );
    });

    it('coalesces overlapping maintenance runs into a single backend call', async () => {
        mocks.isHostCapabilityAvailable.mockReturnValue(true);
        let resolveRun: (value: {
            restorePromptNeeded: boolean;
        }) => void = () => {
            throw new Error('Maintenance run was not started.');
        };
        mocks.appRegistryBackupMaintenanceRun.mockImplementationOnce(
            () =>
                new Promise((resolve) => {
                    resolveRun = resolve;
                })
        );

        const startupRun = runStartupMaintenance();
        const foregroundUpdateRun =
            runForegroundUpdateRegistryBackupMaintenance();
        resolveRun({ restorePromptNeeded: false });
        await Promise.all([startupRun, foregroundUpdateRun]);

        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenCalledTimes(1);
        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenCalledWith(
            'foreground-startup'
        );

        mocks.appRegistryBackupMaintenanceRun.mockResolvedValueOnce({
            restorePromptNeeded: false
        });
        await runForegroundUpdateRegistryBackupMaintenance();

        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenCalledTimes(2);
        expect(mocks.appRegistryBackupMaintenanceRun).toHaveBeenLastCalledWith(
            'foreground-update'
        );
    });
});
