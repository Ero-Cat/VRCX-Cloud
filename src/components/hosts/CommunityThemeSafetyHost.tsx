import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { tauriClient } from '@/platform/tauri/client';
import { stopLocalCommunityThemePreview } from '@/services/communityThemeService';
import { toast } from '@/services/toastService';

export function CommunityThemeSafetyHost(): null {
    const { t } = useTranslation();

    useEffect(() => {
        let disposed = false;
        let unlisten: (() => void) | null = null;

        async function disableThemeFromTray() {
            try {
                await stopLocalCommunityThemePreview();
                toast.add({
                    type: 'success',
                    title: t('view.community_themes.toast.theme_disabled')
                });
            } catch (error) {
                toast.add({
                    type: 'error',
                    title:
                        error instanceof Error
                            ? error.message
                            : t('view.community_themes.toast.disable_failed')
                });
            }
        }

        tauriClient.events
            .subscribe('communityThemeDisableRequested', () => {
                void disableThemeFromTray();
            })
            .then((unsubscribe: () => void) => {
                if (disposed) {
                    unsubscribe();
                    return;
                }
                unlisten = unsubscribe;
            })
            .catch((error: unknown) => {
                console.warn(
                    'Unable to subscribe community theme tray event:',
                    error
                );
            });

        return () => {
            disposed = true;
            unlisten?.();
        };
    }, [t]);

    return null;
}
