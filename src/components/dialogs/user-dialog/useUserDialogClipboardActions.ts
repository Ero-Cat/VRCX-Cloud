import { useTranslation } from 'react-i18next';

import { copyTextToClipboard } from '@/services/clipboardService';
import { openDiscordProfile as openShellDiscordProfile } from '@/services/shellIntegrationService';
import { toast } from '@/services/toastService';

export function useUserDialogClipboardActions() {
    const { t } = useTranslation();

    function copyUserText(text: string, label: string) {
        return copyTextToClipboard(text, {
            successMessage: t('dialog.user.dynamic.value_copied', {
                value: label
            })
        });
    }

    async function openDiscordProfile(discordId: string) {
        try {
            const normalizedDiscordId = discordId.trim();
            if (!normalizedDiscordId) {
                return;
            }
            await openShellDiscordProfile(normalizedDiscordId);
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('dialog.user.toast.failed_to_open_discord_profile')
            });
        }
    }

    return {
        copyUserText,
        openDiscordProfile
    };
}
