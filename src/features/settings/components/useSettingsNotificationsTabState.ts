import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';

import { usePreferencesStore } from '@/state/preferencesStore';

import { useSettingsPageSection } from '../SettingsPageStateContext';

export function useSettingsNotificationsTabState() {
    const { t } = useTranslation();
    const notifications = useSettingsPageSection('notifications');
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            desktopToast: state.desktopToast,
            afkDesktopToast: state.afkDesktopToast,
            desktopNotificationSound: state.desktopNotificationSound,
            notificationDoNotDisturbEndOnGameStart:
                state.notificationDoNotDisturbEndOnGameStart,
            notificationTTS: state.notificationTTS,
            notificationTTSVoiceNative: state.notificationTTSVoiceNative,
            notificationTTSVolume: state.notificationTTSVolume,
            notificationTTSNameMode: state.notificationTTSNameMode,
            notificationTTSNickName: state.notificationTTSNickName
        }))
    );
    const {
        desktopToastOptions,
        notificationTtsOptions,
        notificationTtsNameModeOptions,
        ttsVoices,
        notificationTtsTestVisible,
        notificationTtsTest,
        setDesktopNotificationsDialogOpen,
        setTtsNotificationsDialogOpen,
        saveStringPreference,
        saveBoolPreference,
        savePreferenceValue,
        setIntConfigPreference,
        saveNotificationTtsMode,
        saveNotificationTtsVoice,
        setNotificationTtsTestVisible,
        setNotificationTtsTest,
        speakNotificationTts
    } = notifications;

    return {
        prefs,
        desktopToastOptions,
        notificationTtsOptions,
        notificationTtsNameModeOptions,
        ttsVoices,
        notificationTtsTestVisible,
        notificationTtsTest,
        onOpenDesktopNotificationFiltersDialog: () =>
            setDesktopNotificationsDialogOpen(true),
        onOpenTtsNotificationFiltersDialog: () =>
            setTtsNotificationsDialogOpen(true),
        onDesktopToastChange: (value: string) => {
            saveStringPreference('desktopToast', 'desktopToast', value);
        },
        onAfkDesktopToastChange: (enabled: boolean) => {
            saveBoolPreference('afkDesktopToast', 'afkDesktopToast', enabled);
        },
        onDesktopNotificationSoundChange: (enabled: boolean) => {
            saveBoolPreference(
                'desktopNotificationSound',
                'desktopNotificationSound',
                enabled
            );
        },
        onNotificationDoNotDisturbEndOnGameStartChange: (enabled: boolean) => {
            saveBoolPreference(
                'notificationDoNotDisturbEndOnGameStart',
                'notificationDoNotDisturbEndOnGameStart',
                enabled
            );
        },
        onNotificationTtsModeChange: (value: string) => {
            saveNotificationTtsMode(value);
        },
        onNotificationTtsVoiceChange: (value: string) => {
            saveNotificationTtsVoice(value);
        },
        onNotificationTtsVolumeChange: (value: number) => {
            const volume = Math.min(100, Math.max(0, Math.round(value)));
            savePreferenceValue('notificationTTSVolume', volume, () =>
                setIntConfigPreference('notificationTTSVolume', volume, {
                    min: 0,
                    max: 100,
                    fallback: 100
                })
            );
        },
        onNotificationTtsNameModeChange: (value: string) => {
            saveStringPreference(
                'notificationTTSNameMode',
                'notificationTTSNameMode',
                value
            );
        },
        onNotificationTtsTestVisibleChange: setNotificationTtsTestVisible,
        onNotificationTtsTestChange: setNotificationTtsTest,
        onSpeakNotificationTts: (message: string) =>
            speakNotificationTts(
                message ||
                    t(
                        'view.settings.notifications.notifications.text_to_speech.tts_test_placeholder'
                    )
            )
    };
}
