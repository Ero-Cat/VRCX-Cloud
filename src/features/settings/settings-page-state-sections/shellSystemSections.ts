import { settingsTabs } from '../settingsOptions';
import type { SettingsSectionInput } from '../settingsPageStateSectionTypes';

type ShellSectionInput = SettingsSectionInput<
    'activeSettingsTab' | 'setActiveSettingsTab'
>;

type SystemSectionInput = SettingsSectionInput<
    'savePreferenceValue' | 'setProxyEnabledPreference'
>;

export function buildShellSection({
    activeSettingsTab,
    setActiveSettingsTab
}: ShellSectionInput) {
    return {
        activeSettingsTab,
        setActiveSettingsTab,
        settingsTabs
    };
}

export function buildSystemSection({
    savePreferenceValue,
    setProxyEnabledPreference
}: SystemSectionInput) {
    return {
        savePreferenceValue,
        setProxyEnabledPreference
    };
}
