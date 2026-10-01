import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';

import {
    commands,
    type NotificationWebhookFormat,
    type WebhookDeliverySnapshot
} from '@/platform/tauri/bindings';
import { toast } from '@/services/toastService';
import { usePreferencesStore } from '@/state/preferencesStore';

import { useSettingsPageSection } from '../SettingsPageStateContext';

export function useSettingsNotificationsTabState() {
    const { t } = useTranslation();
    const notifications = useSettingsPageSection('notifications');
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            webhookEnabled: state.webhookEnabled,
            webhookAuthEventsEnabled: state.webhookAuthEventsEnabled,
            webhookUrl: state.webhookUrl,
            webhookFormat: state.webhookFormat,
            webhookFields: state.webhookFields
        }))
    );
    const {
        setWebhookNotificationsDialogOpen,
        setPrefs,
        saveStringPreference,
        saveBoolPreference
    } = notifications;
    const [webhookDeliverySnapshot, setWebhookDeliverySnapshot] =
        useState<WebhookDeliverySnapshot | null>(null);
    const [webhookDeliveryLoading, setWebhookDeliveryLoading] = useState(true);

    const refreshWebhookDeliveryStatus = useCallback(
        async (showError: boolean) => {
            setWebhookDeliveryLoading(true);
            try {
                setWebhookDeliverySnapshot(
                    await commands.appWebhookDeliverySnapshotGet()
                );
            } catch (error: unknown) {
                if (showError) {
                    toast.add({
                        type: 'error',
                        title:
                            error instanceof Error
                                ? error.message
                                : String(error)
                    });
                }
            } finally {
                setWebhookDeliveryLoading(false);
            }
        },
        []
    );

    useEffect(() => {
        void refreshWebhookDeliveryStatus(false);
    }, [refreshWebhookDeliveryStatus]);

    return {
        prefs,
        webhookDeliverySnapshot,
        webhookDeliveryLoading,
        onRefreshDeliveryStatus: () => {
            void refreshWebhookDeliveryStatus(true);
        },
        onOpenWebhookNotificationFiltersDialog: () => {
            setWebhookNotificationsDialogOpen(true);
        },
        onWebhookEnabledChange: (checked: boolean) => {
            saveBoolPreference('webhookEnabled', 'webhookEnabled', checked);
        },
        onWebhookAuthEventsEnabledChange: (checked: boolean) => {
            saveBoolPreference(
                'webhookAuthEventsEnabled',
                'webhookAuthEventsEnabled',
                checked
            );
        },
        onWebhookUrlDraftChange: (value: string) => {
            setPrefs((current) => ({
                ...current,
                webhookUrl: String(value ?? '')
            }));
        },
        onWebhookUrlBlur: (value: string) => {
            saveStringPreference('webhookUrl', 'webhookUrl', value);
        },
        onWebhookFormatChange: (value: NotificationWebhookFormat) => {
            saveStringPreference('webhookFormat', 'webhookFormat', value);
        },
        onWebhookFieldsChange: (value: string) => {
            saveStringPreference('webhookFields', 'webhookFields', value);
        },
        onTestWebhook: () => {
            const webhookFormat =
                prefs.webhookFormat === 'discord' ? 'discord' : 'generic';
            commands
                .appWebhookSendTest(
                    String(prefs.webhookUrl || ''),
                    webhookFormat,
                    String(prefs.webhookFields || '')
                )
                .then((outcome) => {
                    toast.add({
                        type: 'success',
                        title: t(
                            'view.settings.notifications.notifications.webhook.test_sent',
                            { status: outcome.status }
                        )
                    });
                })
                .catch((error: unknown) => {
                    toast.add({
                        type: 'error',
                        title:
                            error instanceof Error
                                ? error.message
                                : String(error)
                    });
                });
        }
    };
}
