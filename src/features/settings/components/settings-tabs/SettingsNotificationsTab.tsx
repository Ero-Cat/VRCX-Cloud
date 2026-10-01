import { SettingsTabContent } from '../SettingsViewParts';
import { useSettingsNotificationsTabState } from '../useSettingsNotificationsTabState';
import { WebhookSettingsGroup } from './WebhookSettingsGroup';

export function SettingsNotificationsTab() {
    const state = useSettingsNotificationsTabState();

    return (
        <SettingsTabContent value="notifications">
            <WebhookSettingsGroup
                prefs={state.prefs}
                onWebhookEnabledChange={state.onWebhookEnabledChange}
                onWebhookAuthEventsEnabledChange={
                    state.onWebhookAuthEventsEnabledChange
                }
                onWebhookUrlDraftChange={state.onWebhookUrlDraftChange}
                onWebhookUrlBlur={state.onWebhookUrlBlur}
                onWebhookFormatChange={state.onWebhookFormatChange}
                onWebhookFieldsChange={state.onWebhookFieldsChange}
                onOpenWebhookNotificationFiltersDialog={
                    state.onOpenWebhookNotificationFiltersDialog
                }
                onTestWebhook={state.onTestWebhook}
                deliverySnapshot={state.webhookDeliverySnapshot}
                deliveryStatusLoading={state.webhookDeliveryLoading}
                onRefreshDeliveryStatus={state.onRefreshDeliveryStatus}
            />
        </SettingsTabContent>
    );
}
