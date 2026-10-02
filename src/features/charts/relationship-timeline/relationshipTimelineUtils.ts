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
    /** userId → bucket index → score (ms). */
    perFriendBuckets: Map<string, Map<number, number>>;
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
            bucketTotals[bucketIndex] += score;
        }
    }

    const xLabels = Array.from({ length: bucketCount }, (_, index) =>
        bucketLabel(index, firstDay, bucketDays)
    );
    return { bucketCount, xLabels, perFriendBuckets, bucketTotals };
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
    /** Percentage share per bucket (0-100), null bucket → 0. */
    data: number[];
}

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
    resolveDisplayName: (userId: string) => string;
    othersName: string;
    colorPalette: string[];
}): TimelineSeries[] {
    const { bucketCount, perFriendBuckets, bucketTotals } = aggregation;

    // Top-N per bucket: rank friends by their share inside each bucket,
    // then keep the friends that appear in the top-N of any bucket most
    // often (jirai semantics: per-bucket top-N, rest folded into others).
    const topSet = new Set<string>();
    for (let bucket = 0; bucket < bucketCount; bucket += 1) {
        const entries: Array<{ userId: string; score: number }> = [];
        for (const [userId, buckets] of perFriendBuckets) {
            const score = buckets.get(bucket) ?? 0;
            if (score > 0) {
                entries.push({ userId, score });
            }
        }
        entries.sort((left, right) => right.score - left.score);
        for (const entry of entries.slice(0, friendCount)) {
            topSet.add(entry.userId);
        }
    }

    const series: TimelineSeries[] = [];
    let hasOthers = false;
    const othersScores = Array.from<number>({ length: bucketCount }).fill(0);
    for (const [userId, buckets] of perFriendBuckets) {
        if (topSet.has(userId)) {
            series.push({
                userId,
                name: resolveDisplayName(userId),
                color: '',
                data: bucketsToPercent(buckets, bucketTotals, bucketCount)
            });
            continue;
        }
        hasOthers = true;
        for (let bucket = 0; bucket < bucketCount; bucket += 1) {
            othersScores[bucket] += buckets.get(bucket) ?? 0;
        }
    }

    // Stable color assignment by total score, descending.
    series.sort(
        (left, right) =>
            totalScore(right.data) - totalScore(left.data) ||
            left.name.localeCompare(right.name)
    );
    series.forEach((item, index) => {
        item.color = colorPalette[index % colorPalette.length];
    });

    if (showOthers && hasOthers) {
        series.push({
            userId: '__others__',
            name: othersName,
            color: '#8b949e',
            data: othersScores.map((scoreMs, index) =>
                bucketTotals[index] > 0
                    ? +((scoreMs / bucketTotals[index]) * 100).toFixed(1)
                    : 0
            )
        });
    }
    return series;
}

function totalScore(data: number[]): number {
    return data.reduce((total, value) => total + value, 0);
}

function bucketsToPercent(
    buckets: Map<number, number>,
    bucketTotals: number[],
    bucketCount: number
): number[] {
    return Array.from({ length: bucketCount }, (_, index) =>
        bucketTotals[index] > 0
            ? +(
                  ((buckets.get(index) ?? 0) / bucketTotals[index]) *
                  100
              ).toFixed(1)
            : 0
    );
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
