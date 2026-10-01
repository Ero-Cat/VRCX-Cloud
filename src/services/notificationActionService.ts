import {
    toBoopEmojiSendParams,
    type BoopEmojiChoice
} from '@/domain/entities/boopEmoji';
import {
    commands,
    type NotificationActionOutcome,
    type NotificationTarget,
    type SocialFriendMutationOutcome
} from '@/platform/tauri/bindings';
import notificationPersistenceRepository, {
    type NotificationResponse,
    type NotificationRow
} from '@/repositories/notificationPersistenceRepository';
import { recordRecentBoopEmoji } from '@/services/boopRecentService';

type NotificationRecord = Partial<
    Pick<
        NotificationRow,
        | 'id'
        | 'version'
        | 'type'
        | 'senderUserId'
        | 'senderUsername'
        | 'expired'
        | 'link'
    >
>;

interface NotificationActionInput {
    currentUserId?: string;
    notification?: NotificationRecord | null;
}

interface FriendRequestNotificationInput {
    notification?: NotificationRecord | null;
    targetUser?: { displayName?: string } | null;
}

interface AcceptRequestInviteInput extends NotificationActionInput {
    instanceId?: string;
    worldId?: string;
}

interface InviteResponseInput extends NotificationActionInput {
    responseSlot: number;
    imageData?: string;
    withUploadTimeout?: (
        promise: Promise<NotificationActionOutcome>
    ) => Promise<NotificationActionOutcome>;
}

interface NotificationResponseInput extends NotificationActionInput {
    response?: Pick<NotificationResponse, 'data' | 'type'> | null;
}

interface BoopReplyInput extends NotificationActionInput {
    emoji?: BoopEmojiChoice | null;
}

function normalizeText(value: unknown): string {
    return typeof value === 'string'
        ? value.trim()
        : String(value ?? '').trim();
}

function requireNotification(
    notification: NotificationRecord | null | undefined
) {
    if (!notification) {
        throw new Error('Notification action requires a notification.');
    }
    return notification;
}

function toNotificationTarget(
    notification: NotificationRecord
): NotificationTarget {
    return {
        id: normalizeText(notification.id),
        version: Number(notification.version) || 0,
        type: normalizeText(notification.type),
        senderUserId: normalizeText(notification.senderUserId)
    };
}

function unwrapNotificationActionOutcome(
    outcome: NotificationActionOutcome
): NotificationActionOutcome {
    if (outcome.status === 'remoteFailed') {
        throw new Error(
            outcome.remoteError || 'VRChat notification request failed'
        );
    }
    if (outcome.status === 'remoteOkLocalFailed') {
        throw new Error(
            outcome.localError || 'Notification local update failed.'
        );
    }
    return outcome;
}

export async function findIncomingFriendRequestNotification({
    currentUserId,
    targetUserId
}: {
    currentUserId?: string;
    targetUserId?: string;
}) {
    const normalizedCurrentUserId = normalizeText(currentUserId);
    const normalizedTargetUserId = normalizeText(targetUserId);
    if (!normalizedCurrentUserId || !normalizedTargetUserId) {
        return null;
    }

    const rows = await notificationPersistenceRepository.queryNotifications({
        userId: normalizedCurrentUserId,
        filters: ['friendRequest']
    });
    return (
        rows.find(
            (row) =>
                row?.type === 'friendRequest' &&
                !row.expired &&
                normalizeText(row.senderUserId) === normalizedTargetUserId
        ) || null
    );
}

export async function expireNotificationLocally({
    currentUserId,
    notification
}: NotificationActionInput) {
    const target = requireNotification(notification);
    await notificationPersistenceRepository.expireNotification({
        userId: currentUserId,
        id: normalizeText(target.id)
    });
}

export async function hideRemoteAndExpireNotification({
    currentUserId,
    notification
}: NotificationActionInput) {
    const target = requireNotification(notification);
    const outcome = await commands.appNotificationHideAndExpire({
        ownerUserId: normalizeText(currentUserId),
        target: toNotificationTarget(target)
    });
    unwrapNotificationActionOutcome(outcome);
}

export async function acceptFriendRequestNotification({
    notification,
    targetUser = null
}: FriendRequestNotificationInput): Promise<
    | { status: 'accepted'; outcome: SocialFriendMutationOutcome }
    | { status: 'not-found' }
> {
    const target = requireNotification(notification);
    const targetUserId = normalizeText(target.senderUserId);
    const targetDisplayName =
        normalizeText(targetUser?.displayName) ||
        normalizeText(target.senderUsername);

    const result = await commands.appSocialFriendRequestNotificationAccept({
        notificationId: normalizeText(target.id),
        targetUserId,
        targetDisplayName
    });
    if (result.status === 'notFound') {
        return { status: 'not-found' };
    }
    if (!result.outcome) {
        throw new Error('Friend request accept result is incomplete.');
    }
    return { status: 'accepted', outcome: result.outcome };
}

export async function acceptRequestInviteNotification({
    currentUserId,
    notification,
    instanceId,
    worldId
}: AcceptRequestInviteInput) {
    const target = requireNotification(notification);
    const notificationTarget = toNotificationTarget(target);
    const normalizedInstanceId = normalizeText(instanceId);
    const outcome = await commands.appNotificationRequestInviteAccept({
        ownerUserId: normalizeText(currentUserId),
        target: notificationTarget,
        instanceId: normalizedInstanceId,
        worldId: normalizeText(worldId)
    });
    unwrapNotificationActionOutcome(outcome);
}

export async function sendInviteResponseNotification({
    currentUserId,
    notification,
    responseSlot,
    imageData,
    withUploadTimeout
}: InviteResponseInput) {
    const target = requireNotification(notification);
    const invoke = () =>
        commands.appNotificationInviteResponseSend({
            ownerUserId: normalizeText(currentUserId),
            target: toNotificationTarget(target),
            responseSlot,
            imageData: imageData?.trim() ?? ''
        });
    const outcome =
        imageData && withUploadTimeout
            ? await withUploadTimeout(invoke())
            : await invoke();
    unwrapNotificationActionOutcome(outcome);
    return { sentPhoto: Boolean(imageData?.trim()) };
}

export async function dismissBoopNotifications({
    currentUserId,
    senderUserId
}: {
    currentUserId?: string;
    senderUserId?: string;
}) {
    const normalizedSenderUserId = normalizeText(senderUserId);
    if (!currentUserId || !normalizedSenderUserId) {
        return;
    }
    await commands.appNotificationBoopDismiss({
        ownerUserId: normalizeText(currentUserId),
        senderUserId: normalizedSenderUserId
    });
}

export async function sendBoopReplyNotification({
    currentUserId,
    notification,
    emoji = null
}: BoopReplyInput) {
    const target = requireNotification(notification);
    const senderUserId = normalizeText(target.senderUserId);
    if (!senderUserId) {
        throw new Error('Cannot send boop: no sender user id is available.');
    }
    const outcome = await commands.appNotificationBoopReply({
        ownerUserId: normalizeText(currentUserId),
        target: toNotificationTarget(target),
        ...toBoopEmojiSendParams(emoji)
    });
    unwrapNotificationActionOutcome(outcome);
    if (emoji) {
        await recordRecentBoopEmoji(emoji).catch(() => {});
    }
}

export async function sendNotificationButtonResponse({
    currentUserId,
    notification,
    response
}: NotificationResponseInput) {
    const target = requireNotification(notification);
    const outcome = await commands.appNotificationRespondAndExpire({
        ownerUserId: normalizeText(currentUserId),
        target: toNotificationTarget(target),
        responseType: normalizeText(response?.type),
        responseData: response?.data || ''
    });
    unwrapNotificationActionOutcome(outcome);
}
