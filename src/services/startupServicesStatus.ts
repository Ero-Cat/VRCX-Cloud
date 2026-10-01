import { useRuntimeStore } from '@/state/runtimeStore';
import { useSessionStore } from '@/state/sessionStore';

type StartupServicesStatus = {
    completed: boolean;
    pending: string[];
    detail: string;
};

function joinPendingItems(items: string[]): string {
    if (items.length === 0) {
        return '';
    }
    if (items.length === 1) {
        return items[0];
    }
    if (items.length === 2) {
        return `${items[0]} and ${items[1]}`;
    }

    return `${items.slice(0, -1).join(', ')}, and ${items[items.length - 1]}`;
}

export function getPendingStartupServices(): string[] {
    const sessionState = useSessionStore.getState();
    const pending: string[] = [];

    if (!sessionState.isFriendsLoaded) {
        pending.push('friend roster baseline');
    }
    if (sessionState.transportStatus !== 'pipeline-connected') {
        pending.push('realtime transport');
    }
    if (!sessionState.isFavoritesLoaded) {
        pending.push('favorites hydration');
    }

    return pending;
}

export function syncStartupServicesTask(
    baseParts: unknown[] = []
): StartupServicesStatus {
    const runtimeStore = useRuntimeStore.getState();
    const currentStartupStatus = runtimeStore.startup.services.status;
    const pending = getPendingStartupServices();
    const completed = pending.length === 0;
    const detailTail = completed
        ? 'Friend roster baseline, realtime transport, and favorites hydration are active.'
        : `Pending: ${joinPendingItems(pending)}.`;
    const detail = [...baseParts.filter(Boolean), detailTail].join(' ');

    runtimeStore.setStartupTask(
        'services',
        currentStartupStatus === 'error'
            ? 'error'
            : completed
              ? 'completed'
              : 'pending',
        detail
    );
    return {
        completed,
        pending,
        detail
    };
}
