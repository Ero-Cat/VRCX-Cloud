import { PersonStandingIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Badge } from '@/ui/shadcn/badge';

import type { AvatarViewRecord } from '../avatarDialogTypes';

export function AvatarDialogHeaderBadges({
    avatar,
    isCurrentAvatar,
    avatarBlocked
}: {
    avatar: AvatarViewRecord;
    isCurrentAvatar: boolean;
    avatarBlocked: boolean;
}) {
    const { t } = useTranslation();

    return (
        <>
            <Badge
                variant={
                    avatar.releaseStatus === 'public' ? 'default' : 'outline'
                }
            >
                {avatar.releaseStatus === 'public'
                    ? t('dialog.avatar.tags.public')
                    : t('dialog.avatar.tags.private')}
            </Badge>
            {isCurrentAvatar ? (
                <Badge variant="secondary">
                    <PersonStandingIcon data-icon="inline-start" />
                    {t('common.current_session')}
                </Badge>
            ) : null}
            {avatarBlocked ? (
                <Badge variant="destructive">
                    {t('dialog.avatar.error.blocked')}
                </Badge>
            ) : null}
        </>
    );
}
