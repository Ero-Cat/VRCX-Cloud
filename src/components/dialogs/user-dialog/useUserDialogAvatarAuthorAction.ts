import { useTranslation } from 'react-i18next';

import avatarProfileRepository from '@/repositories/avatarProfileRepository';
import { openUserDialog } from '@/services/dialogService';
import { toast } from '@/services/toastService';

export function useUserDialogAvatarAuthorAction({
    currentAvatarTarget
}: {
    currentAvatarTarget: string;
}) {
    const { t } = useTranslation();

    return async function showAvatarAuthor() {
        if (!currentAvatarTarget) {
            return;
        }
        try {
            const avatar = await avatarProfileRepository.getAvatarProfile({
                avatarId: currentAvatarTarget
            });
            if (avatar.authorId) {
                openUserDialog({
                    userId: avatar.authorId,
                    title: avatar.authorName || undefined
                });
                return;
            }
            toast.add({
                type: 'error',
                title: t('dialog.user.error.avatar_author_unavailable')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('dialog.user.toast.failed_to_load_avatar_author')
            });
        }
    };
}
