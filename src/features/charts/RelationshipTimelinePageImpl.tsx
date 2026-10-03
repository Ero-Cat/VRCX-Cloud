import type { EChartsType } from 'echarts/core';
import {
    InfoIcon,
    RefreshCcwIcon,
    UsersIcon,
    ZoomInIcon,
    ZoomOutIcon
} from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { PageScaffold } from '@/components/layout/PageScaffold';
import {
    aggregateFriendDaysToBuckets,
    buildPerBucketTopNPercentageSeries,
    computeZoomRange,
    groupRowsByFriend
} from '@/features/charts/relationship-timeline/relationshipTimelineUtils';
import { echarts } from '@/lib/echarts';
import { commands } from '@/platform/native/bindings';
import { getResolvedThemeMode } from '@/services/themeService';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useShellStore } from '@/state/shellStore';
import { Button } from '@/ui/shadcn/button';
import {
    HoverCard,
    HoverCardContent,
    HoverCardTrigger
} from '@/ui/shadcn/hover-card';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue
} from '@/ui/shadcn/select';
import { Slider } from '@/ui/shadcn/slider';
import { Spinner } from '@/ui/shadcn/spinner';
import { Switch } from '@/ui/shadcn/switch';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

const COLOR_PALETTE = [
    '#5470c6',
    '#91cc75',
    '#fac858',
    '#ee6666',
    '#73c0de',
    '#3ba272',
    '#fc8452',
    '#9a60b4',
    '#ea7ccc',
    '#17c0ac'
];
const DEFAULT_VISIBLE_BUCKETS = 10;
/** bucketDays = round(90 ^ (slider / 100)): 1 … 90 days per unit. */
const DEFAULT_SCALE_SLIDER = 51;

export function RelationshipTimelinePage() {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const themeMode = useShellStore((state) => state.themeMode);
    const isDarkMode = getResolvedThemeMode(themeMode) === 'dark';

    const [rows, setRows] = useState<
        | Awaited<
              ReturnType<typeof commands.appRelationshipTimelineRows>
          >['rows']
        | null
    >(null);
    const [loading, setLoading] = useState(false);
    const [friendCountLive, setFriendCountLive] = useState(5);
    const [friendCount, setFriendCount] = useState(5);
    const [showOthers, setShowOthers] = useState(false);
    const [showFriendsOnly, setShowFriendsOnly] = useState(false);
    const [scaleLive, setScaleLive] = useState(DEFAULT_SCALE_SLIDER);
    const [scaleSlider, setScaleSlider] = useState(DEFAULT_SCALE_SLIDER);
    const [rangeDays, setRangeDays] = useState(0);
    const [zoomRange, setZoomRange] = useState<{
        start: number;
        end: number;
    } | null>(null);

    const bucketDays = useMemo(
        () => Math.max(1, Math.round(Math.pow(90, scaleSlider / 100))),
        [scaleSlider]
    );

    const [chartElement, setChartElement] = useState<HTMLDivElement | null>(
        null
    );
    const chartInstanceRef = useRef<EChartsType | null>(null);
    const chartThemeRef = useRef<string | null>(null);
    const resizeObserverRef = useRef<ResizeObserver | null>(null);

    const namesByUserId = useMemo(() => {
        const map = new Map<string, string>();
        for (const row of rows ?? []) {
            if (row.displayName && !map.has(row.userId)) {
                map.set(row.userId, row.displayName);
            }
        }
        return map;
    }, [rows]);

    const rangeStartDay = useMemo(() => {
        if (!rangeDays) {
            return null;
        }
        const cutoff = new Date(Date.now() - rangeDays * 24 * 60 * 60 * 1000);
        const pad = (value: number) => String(value).padStart(2, '0');
        return `${cutoff.getFullYear()}-${pad(cutoff.getMonth() + 1)}-${pad(cutoff.getDate())}`;
    }, [rangeDays]);

    const filteredRows = useMemo(() => {
        if (!rangeStartDay || !rows) {
            return rows ?? [];
        }
        return rows.filter((row) => row.day >= rangeStartDay);
    }, [rows, rangeStartDay]);

    const perFriendDays = useMemo(
        () => groupRowsByFriend(filteredRows),
        [filteredRows]
    );

    const seriesData = useMemo(() => {
        let data = perFriendDays;
        if (showFriendsOnly) {
            data = new Map(
                Array.from(data).filter(([userId]) => friendsById[userId])
            );
        }
        const aggregation = aggregateFriendDaysToBuckets(data, bucketDays);
        if (!aggregation) {
            return null;
        }
        const series = buildPerBucketTopNPercentageSeries({
            aggregation,
            friendCount,
            showOthers,
            resolveDisplayName: (userId, fallback) =>
                friendsById[userId]?.displayName ||
                namesByUserId.get(userId) ||
                fallback ||
                userId,
            othersName: t('view.charts.relationship_timeline.others'),
            colorPalette: COLOR_PALETTE
        });
        if (!series) {
            return null;
        }
        return { aggregation, series };
    }, [
        perFriendDays,
        showFriendsOnly,
        showOthers,
        friendCount,
        bucketDays,
        friendsById,
        namesByUserId,
        t
    ]);

    const hasData = filteredRows.length > 0;

    const option = useMemo(() => {
        if (!seriesData) {
            return null;
        }
        const { aggregation, series } = seriesData;
        const range = computeZoomRange(
            aggregation.bucketCount,
            zoomRange,
            DEFAULT_VISIBLE_BUCKETS
        );
        const axisLabelColor = isDarkMode ? '#bbb' : '#555';
        const legendData = series
            .filter((item, index) => {
                if (item.userId === '__others__') {
                    return true;
                }
                const startIndex = Math.floor(
                    (range.start / 100) * (aggregation.bucketCount - 1)
                );
                const endIndex = Math.ceil(
                    (range.end / 100) * (aggregation.bucketCount - 1)
                );
                void index;
                return item.data.some(
                    (value, bucket) =>
                        bucket >= startIndex && bucket <= endIndex && value > 0
                );
            })
            .map((item) => item.name);
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
                    const rowsHtml = items
                        .map(
                            (item) =>
                                `<div style="display:flex;align-items:center;gap:6px;padding:2px 4px">` +
                                `<span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${item.color};flex-shrink:0"></span>` +
                                `<span style="flex:1;font-size:12px">${item.seriesName}</span>` +
                                `<span style="font-size:12px;font-weight:600;tabular-nums">${(item.value ?? 0).toFixed(1)}%</span>` +
                                `</div>`
                        )
                        .join('');
                    return header + rowsHtml;
                }
            },
            legend: {
                top: 0,
                type: 'scroll' as const,
                height: 66,
                textStyle: {
                    color: isDarkMode ? '#ccc' : '#333',
                    fontSize: 11
                },
                data: legendData
            },
            grid: {
                left: '3%',
                right: '2%',
                top: 82,
                bottom: 80,
                containLabel: true
            },
            xAxis: {
                type: 'category' as const,
                data: aggregation.xLabels,
                boundaryGap: false,
                axisLabel: { rotate: 30, fontSize: 10, color: axisLabelColor },
                axisLine: {
                    lineStyle: { color: isDarkMode ? '#444' : '#ddd' }
                }
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
                    start: range.start,
                    end: range.end,
                    bottom: 5,
                    height: 20,
                    borderColor: 'transparent',
                    fillerColor: isDarkMode
                        ? 'rgba(255,255,255,0.08)'
                        : 'rgba(0,0,0,0.06)',
                    handleStyle: { color: isDarkMode ? '#888' : '#aaa' }
                },
                {
                    type: 'inside' as const,
                    xAxisIndex: [0],
                    start: range.start,
                    end: range.end
                }
            ],
            series: series.map((item) => ({
                name: item.name,
                type: 'line' as const,
                stack: 'total',
                areaStyle: { opacity: 0.75 },
                lineStyle: { width: 0 },
                smooth: true,
                smoothMonotone: 'x' as const,
                symbol: 'none',
                color: item.color,
                emphasis: { focus: 'series' as const },
                data: item.data
            }))
        };
    }, [seriesData, zoomRange, isDarkMode]);

    const loadData = async () => {
        if (!currentUserId) {
            return;
        }
        setLoading(true);
        try {
            const output = await commands.appRelationshipTimelineRows({
                ownerUserId: currentUserId
            });
            setRows(output.rows);
        } catch {
            setRows([]);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void loadData();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [currentUserId]);

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

    useEffect(() => {
        if (!chartElement || !option) {
            return undefined;
        }
        const themeName = isDarkMode ? 'dark' : null;
        if (!chartInstanceRef.current || chartThemeRef.current !== themeName) {
            resizeObserverRef.current?.disconnect();
            chartInstanceRef.current?.dispose();
            const chart = echarts.init(chartElement, themeName || undefined, {
                height: chartElement.clientHeight || 320
            });
            chartThemeRef.current = themeName;
            chartInstanceRef.current = chart;
            resizeObserverRef.current = new ResizeObserver(() =>
                chart.resize()
            );
            resizeObserverRef.current.observe(chartElement);
        }
        const instance = chartInstanceRef.current;
        // The workspace panels settle their widths a few frames after the
        // page mounts; apply once the container has a real size (bounded
        // retry), otherwise the canvas sticks at its initial 0 width.
        let frame = 0;
        let attempts = 0;
        const apply = () => {
            if (chartElement.clientWidth > 0) {
                instance.resize();
                instance.setOption(option, true);
                return;
            }
            if (attempts < 120) {
                attempts += 1;
                frame = requestAnimationFrame(apply);
            } else {
                instance.setOption(option, true);
            }
        };
        frame = requestAnimationFrame(apply);
        return () => cancelAnimationFrame(frame);
    }, [chartElement, option, isDarkMode]);

    useEffect(() => {
        const instance = chartInstanceRef.current;
        if (!instance) {
            return undefined;
        }
        const handler = (...args: unknown[]) => {
            const event = args[0] as
                | { batch?: Array<{ start?: number; end?: number }> }
                | { start?: number; end?: number }
                | undefined;
            const batch = Array.isArray(
                (event as { batch?: unknown } | undefined)?.batch
            )
                ? (event as { batch: Array<{ start?: number; end?: number }> })
                      .batch
                : null;
            const start = batch
                ? batch[0]?.start
                : (event as { start?: number } | undefined)?.start;
            const end = batch
                ? batch[0]?.end
                : (event as { end?: number } | undefined)?.end;
            if (Number.isFinite(start) && Number.isFinite(end)) {
                setZoomRange({ start: start ?? 0, end: end ?? 100 });
            }
        };
        instance.on('datazoom', handler);
        return () => {
            instance.off('datazoom', handler);
        };
    }, [option]);

    return (
        <PageScaffold id="chart" className="flex h-full min-h-0 flex-col p-0">
            <div className="flex min-h-0 flex-1 flex-col pt-4">
                <div className="mt-0 flex flex-wrap items-center justify-between gap-2 px-4">
                    <div className="mb-3 flex items-center gap-2">
                        <span className="shrink-0">
                            {t('view.charts.relationship_timeline.header')}
                        </span>
                        <HoverCard>
                            <HoverCardTrigger
                                render={
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="icon-xs"
                                    />
                                }
                            >
                                <InfoIcon className="ml-1 text-xs opacity-70" />
                            </HoverCardTrigger>
                            <HoverCardContent
                                side="bottom"
                                align="start"
                                className="w-80"
                            >
                                <div className="text-xs">
                                    {t(
                                        'view.charts.relationship_timeline.tips.description'
                                    )}
                                </div>
                            </HoverCardContent>
                        </HoverCard>
                    </div>

                    <div className="mb-3 flex flex-wrap items-center gap-2">
                        <div className="flex h-[30px] items-center justify-between px-0.5">
                            <span className="shrink-0 text-sm">
                                {t(
                                    'view.charts.relationship_timeline.settings.top_friends'
                                )}
                            </span>
                            <div className="ml-3 flex items-center gap-2">
                                <Slider
                                    value={[friendCountLive]}
                                    min={1}
                                    max={10}
                                    step={1}
                                    className="w-28"
                                    onValueChange={(
                                        value: number | readonly number[]
                                    ) =>
                                        setFriendCountLive(
                                            (Array.isArray(value)
                                                ? value[0]
                                                : value) ?? 5
                                        )
                                    }
                                    onValueCommitted={(
                                        value: number | readonly number[]
                                    ) =>
                                        setFriendCount(
                                            (Array.isArray(value)
                                                ? value[0]
                                                : value) ?? 5
                                        )
                                    }
                                />
                                <span className="text-muted-foreground w-4 text-right text-xs tabular-nums">
                                    {friendCountLive}
                                </span>
                            </div>
                        </div>
                        <div className="flex h-[30px] items-center justify-between gap-2 px-0.5">
                            <span className="shrink-0 text-sm">
                                {t(
                                    'view.charts.relationship_timeline.settings.show_others'
                                )}
                            </span>
                            <Switch
                                checked={showOthers}
                                onCheckedChange={setShowOthers}
                            />
                        </div>
                        <div className="flex h-[30px] items-center justify-between gap-2 px-0.5">
                            <span className="shrink-0 text-sm">
                                {t(
                                    'view.charts.relationship_timeline.settings.show_friends_only'
                                )}
                            </span>
                            <Switch
                                checked={showFriendsOnly}
                                onCheckedChange={setShowFriendsOnly}
                            />
                        </div>
                        <div className="flex h-[30px] items-center gap-2 px-0.5">
                            <span className="shrink-0 text-sm">
                                {t(
                                    'view.charts.relationship_timeline.settings.time_range'
                                )}
                            </span>
                            <Select
                                value={String(rangeDays)}
                                onValueChange={(value: string | null) =>
                                    setRangeDays(Number(value ?? 0) || 0)
                                }
                            >
                                <SelectTrigger className="h-7 w-24 text-xs">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {[90, 180, 365, 0].map((days) => (
                                        <SelectItem
                                            key={days}
                                            value={String(days)}
                                        >
                                            {days === 0
                                                ? t(
                                                      'view.charts.relationship_timeline.settings.range_all'
                                                  )
                                                : t(
                                                      'view.charts.relationship_timeline.settings.range_days',
                                                      { count: days }
                                                  )}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button
                                        type="button"
                                        className="mr-1.5 rounded-full"
                                        size="icon"
                                        variant="ghost"
                                        disabled={loading}
                                        onClick={loadData}
                                    />
                                }
                            >
                                {loading ? (
                                    <Spinner className="size-4" />
                                ) : (
                                    <RefreshCcwIcon className="size-4" />
                                )}
                            </TooltipTrigger>
                            <TooltipContent side="top">
                                {t('view.charts.relationship_timeline.refresh')}
                            </TooltipContent>
                        </Tooltip>
                    </div>
                </div>

                {loading && rows === null ? (
                    <div className="mt-[100px] flex items-center justify-center">
                        <RefreshCcwIcon className="text-muted-foreground size-6 animate-spin" />
                    </div>
                ) : !hasData ? (
                    <div className="text-muted-foreground mt-[100px] flex flex-col items-center justify-center gap-2">
                        <UsersIcon className="size-12 opacity-20" />
                        <p>{t('view.charts.relationship_timeline.no_data')}</p>
                    </div>
                ) : (
                    <div className="relative mt-2 flex min-h-0 flex-1 flex-col">
                        <div
                            ref={setChartElement}
                            className="h-full min-h-80 w-full flex-1"
                            role="img"
                            aria-label={t(
                                'view.charts.relationship_timeline.header'
                            )}
                        />
                        <div className="flex items-center justify-end gap-2 px-4 py-1">
                            <ZoomOutIcon className="text-muted-foreground size-3.5 shrink-0" />
                            <input
                                type="range"
                                min={0}
                                max={100}
                                step={1}
                                value={scaleLive}
                                className="accent-primary w-44"
                                onChange={(event) =>
                                    setScaleLive(Number(event.target.value))
                                }
                                onPointerUp={() => setScaleSlider(scaleLive)}
                                onKeyUp={() => setScaleSlider(scaleLive)}
                            />
                            <ZoomInIcon className="text-muted-foreground size-3.5 shrink-0" />
                            <span className="text-muted-foreground w-20 text-right text-xs tabular-nums">
                                {Math.max(
                                    1,
                                    Math.round(Math.pow(90, scaleLive / 100))
                                )}{' '}
                                {t(
                                    'view.charts.relationship_timeline.days_per_unit'
                                )}
                            </span>
                        </div>
                    </div>
                )}
            </div>
        </PageScaffold>
    );
}
