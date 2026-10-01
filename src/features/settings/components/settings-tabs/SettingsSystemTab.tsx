import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';
import { Switch } from '@/ui/shadcn/switch';

import { PrivacyLockSetting } from '../PrivacyLockSetting';
import { SettingsCard } from '../SettingsCard';
import { Field } from '../SettingsField';
import { SettingsTabContent } from '../SettingsViewParts';
import { useSettingsSystemTabState } from '../useSettingsSystemTabState';

type SettingsSystemTabContentProps = {
    proxyEnabled?: boolean;
    proxyServer?: string;
    onProxyEnabledChange: (checked: boolean) => void | Promise<void>;
    onProxySettings: () => void;
};

export function SettingsSystemTab() {
    const state = useSettingsSystemTabState();
    return <SettingsSystemTabContent {...state} />;
}

export function SettingsSystemTabContent({
    proxyEnabled,
    proxyServer,
    onProxyEnabledChange,
    onProxySettings
}: SettingsSystemTabContentProps) {
    const { t } = useTranslation();

    return (
        <SettingsTabContent value="system">
            <SettingsCard
                cardId="system.application"
                title={t('view.settings.general.application.header')}
            >
                <PrivacyLockSetting />
                <Field label={t('view.settings.general.application.proxy')}>
                    <div className="flex flex-wrap items-center justify-end gap-2">
                        <Switch
                            checked={proxyEnabled}
                            onCheckedChange={onProxyEnabledChange}
                        />
                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={onProxySettings}
                        >
                            {proxyServer
                                ? t('prompt.proxy_settings.configure')
                                : t('prompt.proxy_settings.configure_empty')}
                        </Button>
                    </div>
                </Field>
            </SettingsCard>
        </SettingsTabContent>
    );
}
