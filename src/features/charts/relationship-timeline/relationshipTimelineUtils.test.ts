import { describe, expect, it } from 'vitest';

import {
    aggregateFriendDaysToBuckets,
    buildPerBucketTopNPercentageSeries,
    groupRowsByFriend,
    type FriendDayEntry
} from './relationshipTimelineUtils';

function friendEntry(
    userId: string,
    days: Array<[number, number, number]>
): FriendDayEntry {
    return {
        userId,
        displayName: userId,
        days: new Map(
            days.map(([day, ms, joins]) => [
                day,
                { totalTimeMs: ms, joinCount: joins }
            ])
        )
    };
}

describe('buildPerBucketTopNPercentageSeries (jirai semantics)', () => {
    it('keeps each bucket to its own top-N and fills the stack to 100%', () => {
        // Two single-day buckets (bucketDays = 1): bucket 0 belongs to X+Y,
        // bucket 1 belongs to Z. With topN=1 only X and Z get bands.
        const aggregation = aggregateFriendDaysToBuckets(
            new Map([
                ['X', friendEntry('X', [[0, 100, 0]])],
                ['Y', friendEntry('Y', [[0, 50, 0]])],
                ['Z', friendEntry('Z', [[1, 70, 0]])]
            ]),
            1
        );
        const series = buildPerBucketTopNPercentageSeries({
            aggregation: aggregation!,
            friendCount: 1,
            showOthers: false,
            resolveDisplayName: (id) => id,
            othersName: 'others',
            colorPalette: ['#111', '#222']
        });
        expect(series).not.toBeNull();
        const byName = new Map(series!.map((item) => [item.name, item.data]));
        expect(byName.get('X')).toEqual([100, 0]);
        expect(byName.get('Z')).toEqual([0, 100]);
        expect(byName.has('Y')).toBe(false);
        // No "others" band when disabled.
        expect(byName.has('others')).toBe(false);
    });

    it('folds the non-top remainder into others when enabled', () => {
        const aggregation = aggregateFriendDaysToBuckets(
            new Map([
                ['X', friendEntry('X', [[0, 75, 0]])],
                ['Y', friendEntry('Y', [[0, 25, 0]])]
            ]),
            1
        );
        const series = buildPerBucketTopNPercentageSeries({
            aggregation: aggregation!,
            friendCount: 1,
            showOthers: true,
            resolveDisplayName: (id) => id,
            othersName: '其他',
            colorPalette: ['#111']
        });
        const byName = new Map(series!.map((item) => [item.name, item.data]));
        expect(byName.get('X')).toEqual([75]);
        expect(byName.get('其他')).toEqual([25]);
        const stack = series!
            .filter((item) => item.userId !== '__others__')
            .reduce((sum, item) => sum + item.data[0], 0);
        const others = byName.get('其他')![0];
        expect(stack + others).toBeCloseTo(100);
    });

    it('weights each join as one extra minute', () => {
        const grouped = groupRowsByFriend([
            {
                userId: 'A',
                displayName: 'A',
                day: '1970-01-01',
                totalTimeMs: 60_000,
                joinCount: 2
            }
        ]);
        const entry = grouped.get('A')!;
        expect(entry.days.get(0)?.totalTimeMs).toBe(60_000);
        const aggregation = aggregateFriendDaysToBuckets(grouped, 1);
        expect(aggregation!.bucketTotals[0]).toBe(60_000 + 2 * 60_000);
    });
});
