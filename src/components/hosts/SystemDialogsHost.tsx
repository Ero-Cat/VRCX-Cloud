import { lazy } from 'react';

import { type HostCapabilityKey } from '@/services/hostCapabilityService';
import { useRuntimeStore } from '@/state/runtimeStore';

import { DataDirCleanupHost } from './DataDirCleanupHost';
import { MountOnFirstOpen } from './MountOnFirstOpen';
import { ProfileRestoreResultHost } from './ProfileRestoreResultHost';
import { DatabaseMaintenanceDialog } from './system-dialogs/DatabaseMaintenanceDialog';
import { DatabaseUpgradeDialog } from './system-dialogs/DatabaseUpgradeDialog';
import { ProfileBackupDialogs } from './system-dialogs/ProfileBackupDialogs';

const ChangelogDialog = lazy(() =>
    import('./system-dialogs/ChangelogDialog').then((module) => ({
        default: module.ChangelogDialog
    }))
);
const KeyboardShortcutsDialog = lazy(() =>
    import('@/components/keyboard/KeyboardShortcutsDialog').then((module) => ({
        default: module.KeyboardShortcutsDialog
    }))
);
const ProxySettingsDialog = lazy(() =>
    import('@/components/proxy/ProxySettingsDialog').then((module) => ({
        default: module.ProxySettingsDialog
    }))
);

export function SystemDialogsHost() {
    const changelogOpen = useRuntimeStore(
        (state) => state.systemHosts.changelogOpen
    );
    const keyboardShortcutsOpen = useRuntimeStore(
        (state) => state.systemHosts.keyboardShortcutsOpen
    );
    const proxySettingsOpen = useRuntimeStore(
        (state) => state.systemHosts.proxySettingsOpen
    );
    const changelogTargetVersion = useRuntimeStore(
        (state) => state.changelogTargetVersion
    );
    const databaseUpgradeOpen = useRuntimeStore(
        (state) => state.databaseUpgrade.open
    );
    const systemHostDatabaseUpgradeOpen = useRuntimeStore(
        (state) => state.systemHosts.databaseUpgradeOpen
    );
    const setSystemHostOpen = useRuntimeStore(
        (state) => state.setSystemHostOpen
    );
    const setChangelogTargetVersion = useRuntimeStore(
        (state) => state.setChangelogTargetVersion
    );

    return (
        <>
            <ProfileRestoreResultHost />
            <DataDirCleanupHost />
            <MountOnFirstOpen open={changelogOpen}>
                <ChangelogDialog
                    open={changelogOpen}
                    targetVersion={changelogTargetVersion}
                    onOpenChange={(open: boolean) => {
                        setSystemHostOpen('changelogOpen', open);
                        if (!open) {
                            setChangelogTargetVersion('');
                        }
                    }}
                />
            </MountOnFirstOpen>
            <DatabaseUpgradeDialog
                open={databaseUpgradeOpen || systemHostDatabaseUpgradeOpen}
            />
            <DatabaseMaintenanceDialog />
            <ProfileBackupDialogs />
            <MountOnFirstOpen open={keyboardShortcutsOpen}>
                <KeyboardShortcutsDialog
                    open={keyboardShortcutsOpen}
                    onOpenChange={(open: boolean) =>
                        setSystemHostOpen('keyboardShortcutsOpen', open)
                    }
                />
            </MountOnFirstOpen>
            <MountOnFirstOpen open={proxySettingsOpen}>
                <ProxySettingsDialog
                    open={proxySettingsOpen}
                    onOpenChange={(open: boolean) =>
                        setSystemHostOpen('proxySettingsOpen', open)
                    }
                />
            </MountOnFirstOpen>
        </>
    );
}
