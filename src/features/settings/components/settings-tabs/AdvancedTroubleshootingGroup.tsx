import { RefreshCwIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';
import { Spinner } from '@/ui/shadcn/spinner';
import { Switch } from '@/ui/shadcn/switch';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

import { SettingsCard } from '../SettingsCard';
import { Field, SettingsSectionHeading } from '../SettingsField';
import type {
    SettingsAdvancedAction,
    SettingsAdvancedPrefs
} from './settingsAdvancedTypes';

type DiagnosticAction = 'online' | 'tables';

type AdvancedTroubleshootingGroupProps = {
    onLogResourceLoadChange: (checked: boolean) => void;
    onRefreshOnlineVisits: SettingsAdvancedAction;
    onRefreshSqliteTableSizes: SettingsAdvancedAction;
    onUdonExceptionLoggingChange: (checked: boolean) => void;
    onlineVisitCount: number | null;
    prefs: SettingsAdvancedPrefs;
    sqliteTableSizeRows: ReadonlyArray<readonly [string, string]>;
    sqliteTableSizes: Record<string, unknown>;
};

export function AdvancedTroubleshootingGroup({
    onLogResourceLoadChange,
    onRefreshOnlineVisits,
    onRefreshSqliteTableSizes,
    onUdonExceptionLoggingChange,
    onlineVisitCount,
    prefs,
    sqliteTableSizeRows,
    sqliteTableSizes
}: AdvancedTroubleshootingGroupProps) {
    const { t } = useTranslation();
    const [pendingAction, setPendingAction] = useState<DiagnosticAction | null>(
        null
    );
    const hasTableSizes = Object.keys(sqliteTableSizes).length > 0;

    async function runAction(
        actionName: DiagnosticAction,
        action: SettingsAdvancedAction
    ) {
        if (pendingAction) {
            return;
        }
        setPendingAction(actionName);
        try {
            await action();
        } finally {
            setPendingAction(null);
        }
    }

    function renderRefreshButton(
        actionName: DiagnosticAction,
        action: SettingsAdvancedAction,
        label: string
    ) {
        const pending = pendingAction === actionName;
        return (
            <Tooltip>
                <TooltipTrigger
                    render={
                        <Button
                            type="button"
                            variant="outline"
                            size="icon-sm"
                            aria-label={label}
                            disabled={pendingAction !== null}
                            onClick={() => runAction(actionName, action)}
                        >
                            {pending ? (
                                <Spinner />
                            ) : (
                                <RefreshCwIcon data-icon="inline-start" />
                            )}
                        </Button>
                    }
                />
                <TooltipContent>{label}</TooltipContent>
            </Tooltip>
        );
    }

    return (
        <SettingsCard
            cardId="advanced.troubleshooting"
            defaultOpen={false}
            title={t(
                'view.settings.advanced.advanced_ui.troubleshooting.header'
            )}
            description={t(
                'view.settings.advanced.advanced_ui.troubleshooting.description'
            )}
        >
            <SettingsSectionHeading
                title={t('view.settings.general.logging.header')}
            />
            <Field
                label={t(
                    'view.settings.advanced.advanced.cache_debug.udon_exception_logging'
                )}
            >
                <Switch
                    checked={prefs.udonExceptionLogging}
                    onCheckedChange={onUdonExceptionLoggingChange}
                />
            </Field>
            <Field label={t('view.settings.general.logging.resource_load')}>
                <Switch
                    checked={prefs.logResourceLoad}
                    onCheckedChange={onLogResourceLoadChange}
                />
            </Field>
            <SettingsSectionHeading
                title={t(
                    'view.settings.advanced.advanced_ui.troubleshooting.tools'
                )}
            />
            <Field
                label={t(
                    'view.settings.advanced.advanced_ui.troubleshooting.database_usage'
                )}
            >
                {renderRefreshButton(
                    'tables',
                    onRefreshSqliteTableSizes,
                    t(
                        'view.settings.advanced.advanced_ui.troubleshooting.refresh_database_usage'
                    )
                )}
            </Field>
            {hasTableSizes ? (
                <div className="text-muted-foreground grid [grid-template-columns:repeat(auto-fit,minmax(12rem,1fr))] gap-x-6 gap-y-1 rounded-lg border p-3 text-sm">
                    {sqliteTableSizeRows.map(([key, labelKey]) => (
                        <div
                            key={key}
                            className="grid grid-cols-[minmax(0,1fr)_auto] gap-3"
                        >
                            <span>{t(labelKey)}</span>
                            <span className="font-mono">
                                {String(sqliteTableSizes[key] ?? '')}
                            </span>
                        </div>
                    ))}
                </div>
            ) : null}
            <Field
                label={t(
                    'view.settings.advanced.advanced_ui.troubleshooting.online_users'
                )}
            >
                <div className="flex items-center justify-end gap-2">
                    {onlineVisitCount !== null ? (
                        <span className="text-muted-foreground text-sm">
                            {t('view.profile.game_info.user_online', {
                                count: onlineVisitCount
                            })}
                        </span>
                    ) : null}
                    {renderRefreshButton(
                        'online',
                        onRefreshOnlineVisits,
                        t(
                            'view.settings.advanced.advanced_ui.troubleshooting.refresh_online_users'
                        )
                    )}
                </div>
            </Field>
        </SettingsCard>
    );
}
