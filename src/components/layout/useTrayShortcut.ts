import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';

import { commands } from '@/platform/tauri/bindings';
import { toast } from '@/services/toastService';
import { initializeTrayShortcut } from '@/services/trayShortcutService';
import { runAfterRestoringNormalWindow } from '@/services/windowModeService';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useSessionStore } from '@/state/sessionStore';

export function useTrayShortcut(): void {
    const platform = useRuntimeStore(
        (state) => state.hostCapabilities.platform
    );
    const ready = useSessionStore((state) => state.sessionPhase === 'ready');
    const navigate = useNavigate();
    const { t } = useTranslation();

    useEffect(() => {
        if (platform !== 'windows') return;
        let disposed = false;
        void initializeTrayShortcut()
            .then(async () => {
                if (disposed || !ready) return;
                if (await commands.appTakeTrayShortcutStartupFailure()) {
                    toast.add({
                        id: 'tray-shortcut-unavailable',
                        type: 'error',
                        title: t('shortcuts.tray.startup_failed'),
                        actionProps: {
                            children: t('app_menu.settings'),
                            onClick: () =>
                                runAfterRestoringNormalWindow(() =>
                                    navigate('/settings?tab=system')
                                )
                        }
                    });
                }
            })
            .catch((error: unknown) => {
                console.warn(
                    'Failed to initialize the global tray shortcut UI:',
                    error
                );
            });
        return () => {
            disposed = true;
        };
    }, [platform, ready, navigate, t]);
}
