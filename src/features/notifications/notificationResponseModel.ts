import type {
    NotificationResponse,
    NotificationRow
} from '@/repositories/notificationPersistenceRepository';

export function shouldOpenBoopReplyDialog(
    notification: NotificationRow | null | undefined,
    response: NotificationResponse | null | undefined
): boolean {
    const responseType = String(response?.type || '').toLowerCase();
    return (
        notification?.type === 'boop' &&
        (responseType === 'reply' ||
            responseType === 'boop' ||
            response?.icon === 'reply')
    );
}
