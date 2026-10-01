import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';

import { toast } from '@/services/toastService';
import { usePreferencesStore } from '@/state/preferencesStore';
import { useRuntimeStore } from '@/state/runtimeStore';

import { useSettingsPageSection } from '../SettingsPageStateContext';

export function useSettingsSystemTabState() {
    const { t } = useTranslation();
    const system = useSettingsPageSection('system');
    const setSystemHostOpen = useRuntimeStore(
        (state) => state.setSystemHostOpen
    );
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            proxyEnabled: state.proxyEnabled,
            proxyServer: state.proxyServer
        }))
    );
    const { savePreferenceValue, setProxyEnabledPreference } = system;

    return {
        proxyEnabled: prefs.proxyEnabled,
        proxyServer: prefs.proxyServer,
        onProxyEnabledChange: async (enabled: boolean) => {
            const saved = await savePreferenceValue(
                'proxyEnabled',
                enabled,
                () => setProxyEnabledPreference(enabled)
            );
            if (saved) {
                toast.add({
                    type: 'success',
                    title: t('prompt.proxy_settings.saved_restart_required')
                });
            }
        },
        onProxySettings: () => {
            setSystemHostOpen('proxySettingsOpen', true);
        }
    };
}
