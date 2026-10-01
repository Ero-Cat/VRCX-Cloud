import type { GroupMembershipBatchProgress } from '@/platform/tauri/bindings';

interface MyGroupsBatchProgressEvent {
    count: number;
    lastPayload: unknown;
}

function isGroupMembershipBatchProgress(
    value: unknown
): value is GroupMembershipBatchProgress {
    return Boolean(
        value &&
        typeof value === 'object' &&
        'ownerUserId' in value &&
        typeof value.ownerUserId === 'string' &&
        'endpoint' in value &&
        typeof value.endpoint === 'string' &&
        'completed' in value &&
        typeof value.completed === 'number' &&
        'total' in value &&
        typeof value.total === 'number'
    );
}

export function resolveMyGroupsBatchProgress({
    busy,
    currentAuthEndpoint,
    currentUserId,
    event,
    previousEventCount
}: {
    busy: boolean;
    currentAuthEndpoint: string;
    currentUserId: string | null;
    event: MyGroupsBatchProgressEvent | null | undefined;
    previousEventCount: number;
}) {
    const progress = event?.lastPayload;
    if (
        !busy ||
        !event ||
        event.count <= previousEventCount ||
        !isGroupMembershipBatchProgress(progress) ||
        progress.ownerUserId !== currentUserId ||
        progress.endpoint !== currentAuthEndpoint
    ) {
        return null;
    }
    return { current: progress.completed, total: progress.total };
}
