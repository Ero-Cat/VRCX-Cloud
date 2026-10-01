import { Trash2Icon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { commands } from '@/platform/tauri/bindings';
import { toast } from '@/services/toastService';
import { normalizeAvatarAutoCleanupPreference } from '@/shared/constants/settings';
import { Button } from '@/ui/shadcn/button';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue
} from '@/ui/shadcn/select';
import { Switch } from '@/ui/shadcn/switch';

import { BrowseHistoryRetentionField } from '../BrowseHistoryRetentionField';
import { SettingsCard } from '../SettingsCard';
import { Field } from '../SettingsField';
import { SettingsTabContent } from '../SettingsViewParts';
import { useSettingsAdvancedTabState } from '../useSettingsAdvancedTabState';
import { AdvancedTroubleshootingGroup } from './AdvancedTroubleshootingGroup';
import type { SettingsAdvancedModel } from './settingsAdvancedTypes';

type SettingsAdvancedTabContentProps = {
    advanced: SettingsAdvancedModel;
};

function DeepLinkRegistrationField() {
    const { t } = useTranslation();
    const [registered, setRegistered] = useState<boolean | null>();
    const [repairing, setRepairing] = useState(false);

    useEffect(() => {
        let active = true;

        commands
            .appDeepLinkRegistrationStatus()
            .then((status) => {
                if (active) {
                    setRegistered(status);
                }
            })
            .catch(() => {
                if (active) {
                    setRegistered(false);
                }
            });

        return () => {
            active = false;
        };
    }, []);

    if (registered === undefined || registered === null) {
        return null;
    }

    async function repairRegistration() {
        setRepairing(true);
        try {
            const status = await commands.appDeepLinkRegistrationRepair();
            setRegistered(status);
            if (status) {
                toast.add({
                    type: 'success',
                    title: t(
                        'view.settings.advanced.advanced_ui.behavior.deep_link_repair_success'
                    )
                });
            } else {
                toast.add({
                    type: 'error',
                    title: t(
                        'view.settings.advanced.advanced_ui.behavior.deep_link_repair_failed'
                    )
                });
            }
        } catch (error: unknown) {
            toast.add({
                type: 'error',
                title: error instanceof Error ? error.message : String(error)
            });
        } finally {
            setRepairing(false);
        }
    }

    return (
        <Field
            label={t(
                'view.settings.advanced.advanced_ui.behavior.deep_link_registration'
            )}
            description={t(
                registered
                    ? 'view.settings.advanced.advanced_ui.behavior.deep_link_registered'
                    : 'view.settings.advanced.advanced_ui.behavior.deep_link_not_registered'
            )}
        >
            <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={repairing}
                onClick={() => void repairRegistration()}
            >
                {t(
                    'view.settings.advanced.advanced_ui.behavior.deep_link_repair'
                )}
            </Button>
        </Field>
    );
}

export function SettingsAdvancedTab() {
    const state = useSettingsAdvancedTabState();
    return <SettingsAdvancedTabContent advanced={state} />;
}

export function SettingsAdvancedTabContent({
    advanced
}: SettingsAdvancedTabContentProps) {
    const {
        prefs,
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        onFeedPersistenceDisabledChange,
        onAvatarAutoCleanupChange,
        onOpenPurgeDialog,
        onRefreshSqliteTableSizes,
        onRefreshOnlineVisits,
        onLogResourceLoadChange,
        onUdonExceptionLoggingChange
    } = advanced;
    const { t } = useTranslation();

    return (
        <SettingsTabContent value="advanced">
            <SettingsCard
                cardId="advanced.behavior"
                title={t('view.settings.advanced.advanced_ui.behavior.header')}
            >
                <DeepLinkRegistrationField />
            </SettingsCard>

            <SettingsCard
                cardId="advanced.storage"
                title={t('view.settings.advanced.advanced_ui.storage.header')}
            >
                <Field
                    label={t(
                        'view.settings.advanced.advanced_ui.storage.keep_avatar_data'
                    )}
                    description={t(
                        'view.settings.advanced.advanced_ui.storage.avatar_cleanup_description'
                    )}
                    controlId="settings-avatar-auto-cleanup"
                >
                    <Select
                        value={prefs.avatarAutoCleanup}
                        items={avatarAutoCleanupOptions.map((value) => ({
                            value,
                            label:
                                value === 'Off'
                                    ? t(
                                          'view.settings.advanced.advanced.database_cleanup.auto_cleanup_off'
                                      )
                                    : t(
                                          `view.settings.advanced.advanced.database_cleanup.auto_cleanup_${value}`
                                      )
                        }))}
                        onValueChange={(value) =>
                            onAvatarAutoCleanupChange(
                                normalizeAvatarAutoCleanupPreference(value)
                            )
                        }
                    >
                        <SelectTrigger
                            id="settings-avatar-auto-cleanup"
                            className="w-36"
                        >
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectGroup>
                                {avatarAutoCleanupOptions.map((value) => (
                                    <SelectItem key={value} value={value}>
                                        {value === 'Off'
                                            ? t(
                                                  'view.settings.advanced.advanced.database_cleanup.auto_cleanup_off'
                                              )
                                            : t(
                                                  `view.settings.advanced.advanced.database_cleanup.auto_cleanup_${value}`
                                              )}
                                    </SelectItem>
                                ))}
                            </SelectGroup>
                        </SelectContent>
                    </Select>
                </Field>
                <BrowseHistoryRetentionField />
                <Field
                    label={t(
                        'view.settings.advanced.advanced_ui.troubleshooting.feed_history'
                    )}
                    description={t(
                        'view.settings.advanced.advanced_ui.troubleshooting.feed_history_description'
                    )}
                >
                    <Switch
                        checked={!prefs.feedPersistenceDisabled}
                        onCheckedChange={(checked) =>
                            onFeedPersistenceDisabledChange(!checked)
                        }
                    />
                </Field>
            </SettingsCard>

            <AdvancedTroubleshootingGroup
                prefs={prefs}
                sqliteTableSizes={sqliteTableSizes}
                sqliteTableSizeRows={sqliteTableSizeRows}
                onlineVisitCount={onlineVisitCount}
                onRefreshSqliteTableSizes={onRefreshSqliteTableSizes}
                onRefreshOnlineVisits={onRefreshOnlineVisits}
                onLogResourceLoadChange={onLogResourceLoadChange}
                onUdonExceptionLoggingChange={onUdonExceptionLoggingChange}
            />

            {/* Danger zone: destructive, irreversible actions kept visually separate at the bottom. */}
            <section className="border-destructive/30 flex shrink-0 flex-col rounded-lg border">
                <div className="px-4 pt-4 pb-1">
                    <h3 className="text-destructive font-heading text-base leading-snug font-medium">
                        {t('view.settings.advanced_groups.danger.header')}
                    </h3>
                </div>
                <div className="flex flex-col px-4 pb-2">
                    <Field
                        label={t(
                            'view.settings.advanced.advanced_ui.danger.avatar_history'
                        )}
                        description={t(
                            'view.settings.advanced_groups.danger.cannot_be_undone'
                        )}
                    >
                        <Button
                            type="button"
                            variant="destructive"
                            size="sm"
                            onClick={onOpenPurgeDialog}
                        >
                            <Trash2Icon data-icon="inline-start" />
                            {t(
                                'view.settings.advanced.advanced_ui.danger.delete'
                            )}
                        </Button>
                    </Field>
                </div>
            </section>
        </SettingsTabContent>
    );
}
