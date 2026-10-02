import type {
    TwoPersonOverlapRow,
    TwoPersonRelationshipOutput,
    TwoPersonSelfPresenceRow
} from '@/platform/native/bindings';

export type InitiatorKind = 'mutual' | 'leftPlayer' | 'rightPlayer' | 'unknown';

export interface SharedInstanceItem {
    location: string;
    /** Latest leave timestamp across the merged segments (ms). */
    lastLeaveMs: number;
    formattedDate: string;
    coexistenceTimeMs: number;
    joinLeavesCount: number;
    instanceCreatorName: string | null;
    maxPlayerCount: number | null;
    selfPresent: boolean;
    initiator: InitiatorKind;
}

const THREE_MINUTES_MS = 3 * 60 * 1000;
const DAY_MS = 24 * 60 * 60 * 1000;

function parseMs(value: string): number {
    const ms = Date.parse(value);
    return Number.isFinite(ms) ? ms : 0;
}

/** Instance creator id from a `wrld_x:userId(...)` location string. */
export function instanceCreatorIdOf(location: string): string | null {
    const instancePart = location.split(':')[1];
    if (!instancePart) {
        return null;
    }
    const match = instancePart.match(/(usr_[0-9a-fA-F-]+)/);
    return match ? match[1] : null;
}

function formatDateTime(ms: number, hour12: boolean) {
    const date = new Date(ms);
    const pad = (value: number) => String(value).padStart(2, '0');
    const hours = hour12
        ? (() => {
              const h = date.getHours() % 12 || 12;
              return `${pad(h)}:${pad(date.getMinutes())} ${date.getHours() < 12 ? 'AM' : 'PM'}`;
          })()
        : `${pad(date.getHours())}:${pad(date.getMinutes())}`;
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${hours}`;
}

export function buildSharedInstances(
    output: TwoPersonRelationshipOutput,
    resolveName: (userId: string) => string | null,
    hour12: boolean
): SharedInstanceItem[] {
    const rows = output.rows ?? [];
    if (rows.length === 0) {
        return [];
    }
    const selfSessionsByLocation = new Map<
        string,
        TwoPersonSelfPresenceRow[]
    >();
    for (const row of output.selfPresence ?? []) {
        const list = selfSessionsByLocation.get(row.location) ?? [];
        list.push(row);
        selfSessionsByLocation.set(row.location, list);
    }
    const maxCountByLocation = new Map(
        (output.maxPlayerCounts ?? []).map(([location, count]) => [
            location,
            count
        ])
    );

    function selfPresentAt(location: string, start: number, end: number) {
        for (const session of selfSessionsByLocation.get(location) ?? []) {
            const leaveMs = parseMs(session.selfLeave);
            const joinMs = leaveMs - Math.max(0, session.selfTime);
            if (joinMs < end && leaveMs > start) {
                return true;
            }
        }
        return false;
    }

    interface Segment {
        aJoin: number;
        bJoin: number;
        aLeave: number;
        bLeave: number;
    }
    const segmentsByLocation = new Map<string, Segment[]>();
    const firstJoinByLocation = new Map<
        string,
        { aJoin: number; bJoin: number }
    >();
    for (const row of rows) {
        const aLeave = parseMs(row.friendALeave);
        const bLeave = parseMs(row.friendBLeave);
        const aJoin = aLeave - Math.max(0, row.friendATime);
        const bJoin = bLeave - Math.max(0, row.friendBTime);
        const segment: Segment = { aJoin, bJoin, aLeave, bLeave };
        const list = segmentsByLocation.get(row.location) ?? [];
        list.push(segment);
        segmentsByLocation.set(row.location, list);
        const first = firstJoinByLocation.get(row.location);
        if (
            !first ||
            Math.max(aJoin, bJoin) < Math.max(first.aJoin, first.bJoin)
        ) {
            firstJoinByLocation.set(row.location, { aJoin, bJoin });
        }
    }

    const items: SharedInstanceItem[] = [];
    for (const [location, segments] of segmentsByLocation) {
        let coexistenceTimeMs = 0;
        let lastLeaveMs = 0;
        let selfPresent = false;
        for (const segment of segments) {
            const start = Math.max(segment.aJoin, segment.bJoin);
            const end = Math.min(segment.aLeave, segment.bLeave);
            if (end > start) {
                coexistenceTimeMs += end - start;
            }
            lastLeaveMs = Math.max(lastLeaveMs, segment.aLeave, segment.bLeave);
            selfPresent =
                selfPresent ||
                selfPresentAt(location, start, Math.max(end, start));
        }

        const first = firstJoinByLocation.get(location);
        let initiator: InitiatorKind = 'mutual';
        if (first) {
            const joinDiff = Math.abs(first.aJoin - first.bJoin);
            if (joinDiff > THREE_MINUTES_MS) {
                initiator =
                    first.aJoin > first.bJoin ? 'leftPlayer' : 'rightPlayer';
            } else if (selfPresent) {
                // Both joined within the window *and* so did the owner:
                // likely observed together in one snapshot, direction unknown.
                for (const session of selfSessionsByLocation.get(location) ??
                    []) {
                    const myJoin =
                        parseMs(session.selfLeave) -
                        Math.max(0, session.selfTime);
                    if (
                        Math.abs(first.aJoin - myJoin) <= THREE_MINUTES_MS &&
                        Math.abs(first.bJoin - myJoin) <= THREE_MINUTES_MS
                    ) {
                        initiator = 'unknown';
                        break;
                    }
                }
            }
        }

        items.push({
            location,
            lastLeaveMs,
            formattedDate: formatDateTime(lastLeaveMs, hour12),
            coexistenceTimeMs,
            joinLeavesCount: segments.length,
            instanceCreatorName: ((creatorId) =>
                creatorId ? resolveName(creatorId) : null)(
                instanceCreatorIdOf(location)
            ),
            maxPlayerCount: maxCountByLocation.get(location) ?? null,
            selfPresent,
            initiator
        });
    }
    items.sort((left, right) => right.lastLeaveMs - left.lastLeaveMs);
    return items;
}

export function formatDuration(ms: number): string {
    if (ms <= 0) {
        return '0m';
    }
    const totalMinutes = Math.round(ms / 60000);
    const days = Math.floor(totalMinutes / (60 * 24));
    const hours = Math.floor((totalMinutes % (60 * 24)) / 60);
    const minutes = totalMinutes % 60;
    const parts: string[] = [];
    if (days > 0) {
        parts.push(`${days}d`);
    }
    if (hours > 0) {
        parts.push(`${hours}h`);
    }
    if (minutes > 0 || parts.length === 0) {
        parts.push(`${minutes}m`);
    }
    return parts.join(' ');
}

export const SHARED_INSTANCE_DAY_MS = DAY_MS;
export type { TwoPersonOverlapRow };
