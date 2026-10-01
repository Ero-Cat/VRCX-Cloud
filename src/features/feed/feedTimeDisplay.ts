import type { TFunction } from 'i18next';

import { formatDateFilter, formatRelativeTime } from '@/lib/dateTime';
import { dateFromUnknown } from '@/shared/utils/dateTime';
import type { FeedTimeDisplayModePreference } from '@/state/preferencesStore';

type FeedTimestamp = string | null | undefined;

function parseTimestampMs(value: FeedTimestamp) {
    if (!value) {
        return null;
    }

    return dateFromUnknown(value)?.getTime() ?? null;
}

export function formatFeedRelativeTime(
    value: FeedTimestamp,
    nowMs: number,
    _t: TFunction
) {
    const timestampMs = parseTimestampMs(value);
    if (timestampMs === null) {
        return '-';
    }

    return formatRelativeTime(timestampMs, {
        nowMs,
        style: 'short'
    });
}

export function formatFeedExactTime(
    value: FeedTimestamp,
    format: 'short' | 'long' = 'short'
) {
    if (!value) {
        return '-';
    }

    return formatDateFilter(value, format);
}

export function resolveFeedColumnTimeDisplay({
    mode,
    nowMs,
    t,
    value
}: {
    mode: FeedTimeDisplayModePreference;
    nowMs: number;
    t: TFunction;
    value: FeedTimestamp;
}) {
    if (mode === 'relative') {
        return {
            label: formatFeedRelativeTime(value, nowMs, t),
            title: formatFeedExactTime(value, 'long')
        };
    }

    return {
        label: formatFeedExactTime(value, 'short'),
        title: formatFeedRelativeTime(value, nowMs, t)
    };
}
