import { useState } from 'react';

import type {
    NotificationDialogRequest,
    NotificationRow
} from './notificationPageTypes';

export function useNotificationDialogs() {
    const [inviteResponseRequest, setInviteResponseRequest] =
        useState<NotificationDialogRequest>(null);
    const [boopReplyRequest, setBoopReplyRequest] =
        useState<NotificationRow | null>(null);

    return {
        boopReplyRequest,
        inviteResponseRequest,
        setBoopReplyRequest,
        setInviteResponseRequest
    };
}
