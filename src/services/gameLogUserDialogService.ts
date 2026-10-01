import type { TFunction } from 'i18next';

import { openUserDialog } from '@/services/dialogService';
import { toast } from '@/services/toastService';
import { resolveUserByDisplayName } from '@/services/userIdentityService';
import { normalizeString as normalizeId } from '@/shared/utils/string';

type GameLogUserTarget = {
    userId?: string | null;
    displayName?: string | null;
};

function recordOrNull(value: unknown): Record<string, unknown> | null {
    return value && typeof value === 'object' ? { ...value } : null;
}

export async function openGameLogUser(
    row: GameLogUserTarget | null | undefined,
    t: TFunction
) {
    const userId = normalizeId(row?.userId);
    const displayName = normalizeId(row?.displayName);
    if (userId) {
        openUserDialog({ userId, title: displayName || undefined });
        return;
    }
    if (!displayName) {
        return;
    }

    try {
        const resolved = await resolveUserByDisplayName(displayName, {
            search: !displayName.startsWith('ID:')
        });
        if (resolved?.userId) {
            openUserDialog({
                userId: resolved.userId,
                title: resolved.title || displayName,
                seedData: recordOrNull(resolved.seedData)
            });
            return;
        }

        toast.add({
            type: 'info',
            title: t('view.game_log.dynamic.no_user_id_was_found_for_value', {
                value: displayName
            })
        });
    } catch (error) {
        toast.add({
            type: 'error',
            title:
                error instanceof Error
                    ? error.message
                    : t('view.game_log.toast.failed_to_look_up_value', {
                          value: displayName
                      })
        });
    }
}
