import type { EChartsType } from 'echarts/core';
import { RefreshCwIcon, ZoomInIcon, ZoomOutIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { EntityDialogTabContent } from '@/components/dialogs/EntityDialogScaffold';
import { echarts } from '@/lib/echarts';
import type { StatusStatsDailyBucket } from '@/platform/native/bindings';
import { socialAnalyticsService } from '@/services/socialAnalyticsService';
import { getResolvedThemeMode } from '@/services/themeService';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useShellStore } from '@/state/shellStore';
import { Button } from '@/ui/shadcn/button';
import { Spinner } from '@/ui/shadcn/spinner';

import type { UserDialogProfileRecord } from '../useUserDialogProfileResource';

/** VRChat status keys in display order, official light colors. */
const STATUS_KEYS = ['join me', 'active', 'ask me', 'busy'] as const;
const STATUS_COLORS: Record<string, string> = {
    active: '#2ED319',
    'join me': '#00B8FF',
    'ask me': '#E97C03',
    busy: '#C80928'
};
const STATUS_LABEL_KEYS: Record<string, string> = {
    'join me': 'dialog.user.status_distribution.status.join_me',
    active: 'dialog.user.status_distribution.status.active',
    'ask me': 'dialog.user.status_distribution.status.ask_me',
    busy: 'dialog.user.status_distribution.status.busy'
};
const DAY_MS = 24 * 60 * 60 * 1000;
const DEFAULT_VISIBLE_BUCKETS = 10;

function epochDay(day: string): number {
    const ms = Date.parse(`${day}T00:00:00Z`);
    return Number.isFinite(ms) ? Math.floor(ms / DAY_MS) : 0;
}

function bucketLabel(
    firstDay: number,
    bucketIndex: number,
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

/**
 * Re-bucket the per-day minute rows into N-day buckets and normalize to a
 * percentage share per bucket (status minutes / tracked minutes).
 */
function buildPercentageBuckets(
    daily: StatusStatsDailyBucket[],
    bucketDays: number
) {
    const byDay = new Map<string, Map<string, number>>();
    for (const row of daily) {
        let statuses = byDay.get(row.day);
        if (!statuses) {
            statuses = new Map();
            byDay.set(row.day, statuses);
        }
        statuses.set(row.status, (statuses.get(row.status) ?? 0) + row.minutes);
    }
    const days = Array.from(byDay.keys()).sort();
    if (days.length === 0) {
        return null;
    }
    const firstDay = epochDay(days[0]);
    const lastDay = epochDay(days[days.length - 1]);
    const bucketCount = Math.floor((lastDay - firstDay) / bucketDays) + 1;
    const buckets = Array.from({ length: bucketCount }, () => ({
        totals: new Map<string, number>(),
        total: 0
    }));
    for (const [day, statuses] of byDay) {
        const bucketIndex = Math.floor((epochDay(day) - firstDay) / bucketDays);
        const bucket = buckets[bucketIndex];
        if (!bucket) {
            continue;
        }
        for (const [status, minutes] of statuses) {
            bucket.totals.set(
                status,
                (bucket.totals.get(status) ?? 0) + minutes
            );
            bucket.total += minutes;
        }
    }
    const xLabels = Array.from({ length: bucketCount }, (_, index) =>
        bucketLabel(firstDay, index, bucketDays)
    );
    return { buckets, xLabels, bucketCount };
}

export function UserDialogStatusTab({
    profile,
    active
}: {
    profile: UserDialogProfileRecord;
    active: boolean;
}) {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const themeMode = useShellStore((state) => state.themeMode);
    const isDarkMode = getResolvedThemeMode(themeMode) === 'dark';
    const [daily, setDaily] = useState<StatusStatsDailyBucket[]>([]);
    const [loading, setLoading] = useState(false);
    const [loadedFor, setLoadedFor] = useState('');
    const [scaleSlider, setScaleSlider] = useState(51);
    const requestIdRef = useRef(0);
    const userId = profile?.id || '';
    const bucketDays = useMemo(
        () => Math.max(1, Math.round(Math.pow(90, scaleSlider / 100))),
        [scaleSlider]
    );

    const reload = async () => {
        if (!userId || !currentUserId) {
            return;
        }
        const requestId = ++requestIdRef.current;
        setLoading(true);
        try {
            const output = await socialAnalyticsService.loadStatusStats({
                ownerUserId: currentUserId,
                targetUserId: userId
            });
            if (requestId !== requestIdRef.current) {
                return;
            }
            setDaily(output.daily ?? []);
            setLoadedFor(`${currentUserId}:${userId}`);
        } catch {
            if (requestId !== requestIdRef.current) {
                return;
            }
            setDaily([]);
            setLoadedFor(`${currentUserId}:${userId}`);
        } finally {
            if (requestId === requestIdRef.current) {
                setLoading(false);
            }
        }
    };

    useEffect(() => {
        if (!active || !userId || !currentUserId) {
            return undefined;
        }
        const contextKey = `${currentUserId}:${userId}`;
        if (loadedFor === contextKey) {
            return undefined;
        }
        void reload();
        return () => {
            requestIdRef.current += 1;
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [active, currentUserId, userId, loadedFor]);

    const hasData = daily.length > 0;

    const [chartElement, setChartElement] = useState<HTMLDivElement | null>(
        null
    );
    const chartInstanceRef = useRef<EChartsType | null>(null);
    const chartThemeRef = useRef<string | null>(null);
    const resizeObserverRef = useRef<ResizeObserver | null>(null);

    useEffect(
        () => () => {
            resizeObserverRef.current?.disconnect();
            chartInstanceRef.current?.dispose();
            resizeObserverRef.current = null;
            chartInstanceRef.current = null;
            chartThemeRef.current = null;
        },
        []
    );

    const option = useMemo(() => {
        if (daily.length === 0) {
            return null;
        }
        const bucketed = buildPercentageBuckets(daily, bucketDays);
        if (!bucketed) {
            return null;
        }
        const series = STATUS_KEYS.map((status) => ({
            name: t(STATUS_LABEL_KEYS[status]),
            type: 'line' as const,
            stack: 'total',
            areaStyle: { opacity: 0.75 },
            lineStyle: { width: 0 },
            smooth: false,
            symbol: 'none',
            color: STATUS_COLORS[status],
            emphasis: { focus: 'series' as const },
            data: bucketed.buckets.map((bucket) =>
                bucket.total > 0
                    ? +(
                          ((bucket.totals.get(status) ?? 0) / bucket.total) *
                          100
                      ).toFixed(1)
                    : 0
            )
        }));
        const zoomStart =
            bucketed.bucketCount <= DEFAULT_VISIBLE_BUCKETS
                ? 0
                : ((bucketed.bucketCount - DEFAULT_VISIBLE_BUCKETS) /
                      bucketed.bucketCount) *
                  100;
        const axisLabelColor = isDarkMode ? '#bbb' : '#555';
        return {
            backgroundColor: 'transparent',
            tooltip: {
                trigger: 'axis' as const,
                axisPointer: {
                    type: 'line' as const,
                    lineStyle: {
                        color: isDarkMode
                            ? 'rgba(255,255,255,0.3)'
                            : 'rgba(0,0,0,0.2)'
                    }
                },
                formatter(
                    params: Array<{
                        axisValue?: string;
                        color?: string;
                        seriesName?: string;
                        value?: number;
                    }>
                ) {
                    const header = `<div style="margin-bottom:6px;font-weight:600;font-size:12px">${params[0]?.axisValue ?? ''}</div>`;
                    const items = params
                        .filter((item) => (item.value ?? 0) > 0)
                        .sort(
                            (left, right) =>
                                (right.value ?? 0) - (left.value ?? 0)
                        );
                    if (!items.length) {
                        return '';
                    }
                    const rows = items
                        .map(
                            (item) =>
                                `<div style="display:flex;align-items:center;gap:6px;padding:2px 4px">` +
                                `<span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${item.color};flex-shrink:0"></span>` +
                                `<span style="flex:1;font-size:12px">${item.seriesName}</span>` +
                                `<span style="font-size:12px;font-weight:600;tabular-nums">${(item.value ?? 0).toFixed(1)}%</span>` +
                                `</div>`
                        )
                        .join('');
                    return header + rows;
                }
            },
            legend: {
                top: 0,
                type: 'scroll' as const,
                textStyle: { color: isDarkMode ? '#ccc' : '#333', fontSize: 11 }
            },
            grid: {
                left: '3%',
                right: '2%',
                top: 40,
                bottom: 46,
                containLabel: true
            },
            xAxis: {
                type: 'category' as const,
                data: bucketed.xLabels,
                boundaryGap: false,
                axisLabel: { rotate: 30, fontSize: 10, color: axisLabelColor },
                axisLine: { lineStyle: { color: isDarkMode ? '#444' : '#ddd' } }
            },
            yAxis: {
                type: 'value' as const,
                max: 100,
                axisLabel: {
                    formatter: '{value}%',
                    color: axisLabelColor,
                    fontSize: 11
                },
                splitLine: {
                    lineStyle: { color: isDarkMode ? '#333' : '#eee' }
                }
            },
            dataZoom: [
                {
                    type: 'slider' as const,
                    show: true,
                    xAxisIndex: [0],
                    start: zoomStart,
                    end: 100,
                    bottom: 2,
                    height: 16,
                    borderColor: 'transparent',
                    fillerColor: isDarkMode
                        ? 'rgba(255,255,255,0.08)'
                        : 'rgba(0,0,0,0.06)',
                    handleStyle: { color: isDarkMode ? '#888' : '#aaa' }
                },
                {
                    type: 'inside' as const,
                    xAxisIndex: [0],
                    start: zoomStart,
                    end: 100
                }
            ],
            series
        };
    }, [daily, bucketDays, isDarkMode, t]);

    useEffect(() => {
        if (!chartElement || !option) {
            return undefined;
        }
        const themeName = isDarkMode ? 'dark' : null;
        if (!chartInstanceRef.current || chartThemeRef.current !== themeName) {
            resizeObserverRef.current?.disconnect();
            chartInstanceRef.current?.dispose();
            const chart = echarts.init(chartElement, themeName || undefined, {
                height: chartElement.clientHeight || 280
            });
            chartThemeRef.current = themeName;
            chartInstanceRef.current = chart;
            resizeObserverRef.current = new ResizeObserver(() =>
                chart.resize()
            );
            resizeObserverRef.current.observe(chartElement);
        }
        chartInstanceRef.current.setOption(option, true);
        return undefined;
    }, [chartElement, option, isDarkMode]);

    return (
        <EntityDialogTabContent
            value="status"
            className="flex min-w-0 flex-col"
        >
            <div className="flex items-center justify-between">
                <Button
                    type="button"
                    className="rounded-full"
                    variant="ghost"
                    size="icon-sm"
                    disabled={loading}
                    aria-label={t(
                        'dialog.user.status_distribution.refresh_hint'
                    )}
                    onClick={reload}
                >
                    {loading ? <Spinner /> : <RefreshCwIcon />}
                </Button>
                {hasData ? (
                    <div className="flex items-center gap-2 pr-1">
                        <ZoomOutIcon className="text-muted-foreground size-3.5 shrink-0" />
                        <input
                            type="range"
                            min={0}
                            max={100}
                            step={1}
                            value={scaleSlider}
                            className="accent-primary w-28"
                            onChange={(event) =>
                                setScaleSlider(Number(event.target.value))
                            }
                        />
                        <ZoomInIcon className="text-muted-foreground size-3.5 shrink-0" />
                        <span className="text-muted-foreground w-20 text-right text-xs tabular-nums">
                            {bucketDays}{' '}
                            {t('dialog.user.status_distribution.days_per_unit')}
                        </span>
                    </div>
                ) : null}
            </div>

            {loading && !hasData ? (
                <div className="text-muted-foreground mt-8 flex flex-1 flex-col items-center justify-center gap-2 text-sm">
                    <Spinner className="size-5" />
                    <span>{t('dialog.user.status_distribution.loading')}</span>
                </div>
            ) : !hasData ? (
                <div className="text-muted-foreground mt-8 flex max-w-sm flex-1 flex-col items-center justify-center gap-1 px-4 text-center text-sm">
                    <span>{t('dialog.user.status_distribution.no_data')}</span>
                    <span className="text-xs">
                        {t('dialog.user.status_distribution.no_data_hint')}
                    </span>
                </div>
            ) : (
                <div
                    ref={setChartElement}
                    className="min-w-0 flex-1"
                    style={{ width: '100%', minHeight: 280 }}
                    role="img"
                    aria-label={t('dialog.user.status_distribution.header')}
                />
            )}
        </EntityDialogTabContent>
    );
}
