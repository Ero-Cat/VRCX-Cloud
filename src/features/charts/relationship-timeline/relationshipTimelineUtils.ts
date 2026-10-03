import type { RelationshipTimelineDayRow } from '@/platform/native/bindings';

export const DAY_MS = 24 * 60 * 60 * 1000;
/** Each shared join counts as one extra minute of relationship time. */
const JOIN_WEIGHT_MS = 60 * 1000;

export interface FriendDayEntry {
    userId: string;
    displayName: string;
    days: Map<number, { totalTimeMs: number; joinCount: number }>;
}

/** Group raw per-day rows into a per-friend day map (epoch day keyed). */
export function groupRowsByFriend(
    rows: RelationshipTimelineDayRow[]
): Map<string, FriendDayEntry> {
    const map = new Map<string, FriendDayEntry>();
    for (const row of rows) {
        const epochDay = Math.floor(
            Date.parse(`${row.day}T00:00:00Z`) / DAY_MS
        );
        if (!Number.isFinite(epochDay)) {
            continue;
        }
        let entry = map.get(row.userId);
        if (!entry) {
            entry = {
                userId: row.userId,
                displayName: row.displayName,
                days: new Map()
            };
            map.set(row.userId, entry);
        }
        const prev = entry.days.get(epochDay) ?? {
            totalTimeMs: 0,
            joinCount: 0
        };
        entry.days.set(epochDay, {
            totalTimeMs: prev.totalTimeMs + row.totalTimeMs,
            joinCount: prev.joinCount + row.joinCount
        });
    }
    return map;
}

export interface TimelineAggregation {
    bucketCount: number;
    xLabels: string[];
    /** userId → per-bucket score (ms). */
    perFriendBuckets: Map<string, Map<number, number>>;
    /** Per-bucket (userId, score) entries. */
    bucketEntries: Array<Array<{ userId: string; score: number }>>;
    bucketTotals: number[];
}

export function aggregateFriendDaysToBuckets(
    friendDays: Map<string, FriendDayEntry>,
    bucketDays: number
): TimelineAggregation | null {
    if (friendDays.size === 0) {
        return null;
    }
    let firstDay = Number.POSITIVE_INFINITY;
    let lastDay = Number.NEGATIVE_INFINITY;
    for (const entry of friendDays.values()) {
        for (const day of entry.days.keys()) {
            if (day < firstDay) {
                firstDay = day;
            }
            if (day > lastDay) {
                lastDay = day;
            }
        }
    }
    if (!Number.isFinite(firstDay)) {
        return null;
    }

    const bucketCount = Math.floor((lastDay - firstDay) / bucketDays) + 1;
    const perFriendBuckets = new Map<string, Map<number, number>>();
    const bucketEntries: Array<Array<{ userId: string; score: number }>> =
        Array.from({ length: bucketCount }, () => []);
    const bucketTotals = Array.from<number>({ length: bucketCount }).fill(0);

    for (const [userId, entry] of friendDays) {
        const buckets = new Map<number, number>();
        for (const [day, value] of entry.days) {
            const bucketIndex = Math.floor((day - firstDay) / bucketDays);
            const score = value.totalTimeMs + value.joinCount * JOIN_WEIGHT_MS;
            buckets.set(bucketIndex, (buckets.get(bucketIndex) ?? 0) + score);
        }
        perFriendBuckets.set(userId, buckets);
        for (const [bucketIndex, score] of buckets) {
            bucketEntries[bucketIndex].push({ userId, score });
            bucketTotals[bucketIndex] += score;
        }
    }

    const xLabels = Array.from({ length: bucketCount }, (_, index) =>
        bucketLabel(index, firstDay, bucketDays)
    );
    return {
        bucketCount,
        xLabels,
        perFriendBuckets,
        bucketEntries,
        bucketTotals
    };
}

function bucketLabel(
    bucketIndex: number,
    firstDay: number,
    bucketDays: number
) {
    const startDay = firstDay + bucketIndex * bucketDays;
    const start = new Date(startDay * DAY_MS).toISOString().slice(0, 10);
    if (bucketDays === 1) {
        return start;
    }
    const end = new Date((startDay + bucketDays - 1) * DAY_MS)
        .toISOString()
        .slice(0, 10);
    return `${start}~${end}`;
}

export interface TimelineSeries {
    name: string;
    userId: string;
    color: string;
    /** Percentage share per bucket (0-100). */
    data: number[];
}

/**
 * VRCX-jirai parity: each bucket independently keeps its own top-N friends;
 * a friend's series only scores in buckets where it made that bucket's
 * top-N, the remainder folds into "others", and the percentage denominator
 * is the shown total (top sum + others when enabled) so the stack always
 * fills 100%.
 */
export function buildPerBucketTopNPercentageSeries({
    aggregation,
    friendCount,
    showOthers,
    resolveDisplayName,
    othersName,
    colorPalette
}: {
    aggregation: TimelineAggregation;
    friendCount: number;
    showOthers: boolean;
    resolveDisplayName: (userId: string, fallback: string) => string;
    othersName: string;
    colorPalette: string[];
}): TimelineSeries[] | null {
    const { bucketCount, perFriendBuckets, bucketEntries, bucketTotals } =
        aggregation;
    if (!bucketCount) {
        return null;
    }

    const topN = Math.max(1, Math.min(friendCount, perFriendBuckets.size));
    const unionFriendIds = new Set<string>();
    const friendRawData = new Map<string, number[]>();
    const friendSelectedTotals = new Map<string, number>();
    const othersRawData = Array.from<number>({ length: bucketCount }).fill(0);
    const totalsPerBucket = Array.from<number>({ length: bucketCount }).fill(0);

    for (let bucket = 0; bucket < bucketCount; bucket += 1) {
        const sorted = [...bucketEntries[bucket]].sort(
            (left, right) => right.score - left.score
        );
        const topEntries = sorted.slice(0, topN);
        let topSum = 0;
        for (const entry of topEntries) {
            topSum += entry.score;
            unionFriendIds.add(entry.userId);
            if (!friendRawData.has(entry.userId)) {
                friendRawData.set(
                    entry.userId,
                    Array.from<number>({ length: bucketCount }).fill(0)
                );
            }
            (friendRawData.get(entry.userId) as number[])[bucket] = entry.score;
            friendSelectedTotals.set(
                entry.userId,
                (friendSelectedTotals.get(entry.userId) ?? 0) + entry.score
            );
        }
        const othersScore = bucketTotals[bucket] - topSum;
        if (othersScore > 0) {
            othersRawData[bucket] = othersScore;
        }
        totalsPerBucket[bucket] =
            topSum + (showOthers ? othersRawData[bucket] : 0);
    }

    const friendOrder = Array.from(unionFriendIds).sort((left, right) => {
        const scoreDiff =
            (friendSelectedTotals.get(right) ?? 0) -
            (friendSelectedTotals.get(left) ?? 0);
        if (scoreDiff !== 0) {
            return scoreDiff;
        }
        return left.localeCompare(right);
    });

    const toPercent = (value: number, bucket: number): number => {
        const total = totalsPerBucket[bucket];
        if (!total) {
            return 0;
        }
        return Number.parseFloat(((value / total) * 100).toFixed(2));
    };

    const series: TimelineSeries[] = [];
    const includeOthers =
        showOthers && othersRawData.some((value) => value > 0);
    if (includeOthers) {
        series.push({
            name: othersName,
            userId: '__others__',
            color: '#aaaaaa',
            data: othersRawData.map((value, bucket) => toPercent(value, bucket))
        });
    }

    // Stack from lowest to highest selected score: the most significant
    // friend ends up as the top band, matching jirai's push order.
    for (
        let orderIndex = friendOrder.length - 1;
        orderIndex >= 0;
        orderIndex -= 1
    ) {
        const userId = friendOrder[orderIndex];
        series.push({
            name: resolveDisplayName(userId, ''),
            userId,
            color: colorPalette[orderIndex % colorPalette.length],
            data: (friendRawData.get(userId) as number[]).map((value, bucket) =>
                toPercent(value, bucket)
            )
        });
    }
    return series;
}

export function computeZoomRange(
    bucketCount: number,
    persistedRange: { start: number; end: number } | null,
    defaultUnits: number
): { start: number; end: number } {
    if (
        persistedRange &&
        Number.isFinite(persistedRange.start) &&
        Number.isFinite(persistedRange.end)
    ) {
        return {
            start: Math.min(100, Math.max(0, persistedRange.start)),
            end: Math.min(100, Math.max(0, persistedRange.end))
        };
    }
    if (!bucketCount || bucketCount <= defaultUnits) {
        return { start: 0, end: 100 };
    }
    return {
        start: ((bucketCount - defaultUnits) / bucketCount) * 100,
        end: 100
    };
}
