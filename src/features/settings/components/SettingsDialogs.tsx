import { useSettingsPageSection } from '../SettingsPageStateContext';
import { PurgeConfirmDialog } from './settings-dialogs/PurgeConfirmDialog';
import { TableLimitsDialog } from './settings-dialogs/TableLimitsDialog';
import { WebhookNotificationsDialog } from './settings-dialogs/WebhookNotificationsDialog';
import { TablePageSizesDialog } from './SettingsViewParts';

export function SettingsDialogs() {
    const dialogs = useSettingsPageSection('dialogs');
    const tablePageSizes = {
        open: dialogs.tablePageSizesDialogOpen,
        setOpen: dialogs.setTablePageSizesDialogOpen,
        onSaved: (tablePageSizes: number[]) =>
            dialogs.setPrefs((current) => ({ ...current, tablePageSizes }))
    };
    const tableLimits = {
        open: dialogs.tableLimitsDialogOpen,
        setOpen: dialogs.setTableLimitsDialogOpen,
        draft: dialogs.tableLimitsDraft,
        setDraft: dialogs.setTableLimitsDraft,
        tableMaxSizeError: dialogs.tableMaxSizeError,
        searchLimitError: dialogs.searchLimitError,
        saveDisabled: dialogs.tableLimitsSaveDisabled,
        onSave: dialogs.saveTableLimitsDialog
    };
    const purge = {
        open: dialogs.purgeDialogOpen,
        setOpen: dialogs.setPurgeDialogOpen,
        period: dialogs.purgePeriod,
        setPeriod: dialogs.setPurgePeriod,
        inProgress: dialogs.purgeInProgress,
        onConfirm: dialogs.purgeAvatarFeedData
    };
    const webhookNotifications = {
        open: dialogs.webhookNotificationsDialogOpen,
        setOpen: dialogs.setWebhookNotificationsDialogOpen,
        value: dialogs.webhookActivityFilters,
        onSave: dialogs.saveWebhookActivityFilters
    };
    return (
        <>
            <TablePageSizesDialog
                open={tablePageSizes.open}
                onOpenChange={tablePageSizes.setOpen}
                onSaved={tablePageSizes.onSaved}
            />
            <TableLimitsDialog
                open={tableLimits.open}
                onOpenChange={tableLimits.setOpen}
                draft={tableLimits.draft}
                onDraftChange={tableLimits.setDraft}
                tableMaxSizeError={tableLimits.tableMaxSizeError}
                searchLimitError={tableLimits.searchLimitError}
                saveDisabled={tableLimits.saveDisabled}
                onSave={tableLimits.onSave}
            />
            <PurgeConfirmDialog
                open={purge.open}
                onOpenChange={purge.setOpen}
                period={purge.period}
                onPeriodChange={purge.setPeriod}
                inProgress={purge.inProgress}
                onConfirm={purge.onConfirm}
            />
            <WebhookNotificationsDialog
                open={webhookNotifications.open}
                onOpenChange={webhookNotifications.setOpen}
                value={webhookNotifications.value}
                onSave={webhookNotifications.onSave}
            />
        </>
    );
}
