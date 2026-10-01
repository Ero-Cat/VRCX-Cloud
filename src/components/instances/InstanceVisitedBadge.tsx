import { useTranslation } from 'react-i18next';

import { instanceLocationKey } from '@/domain/presence/instancePresence';
import { formatDateFilter } from '@/lib/dateTime';
import { useNowMs } from '@/lib/useNowMs';
import { cn } from '@/lib/utils';
import { HOUR_MS, MINUTE_MS } from '@/shared/constants/time';
import { useInstanceJoinHistoryStore } from '@/state/instanceJoinHistoryStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Badge } from '@/ui/shadcn/badge';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

const VISITED_BADGE_TTL_MS = 3 * HOUR_MS;

function VisitedAgoBadge({
    lastJoinedAtMs,
    className
}: {
    lastJoinedAtMs: number;
    className?: string;
}) {
    const { t } = useTranslation();
    const nowMs = useNowMs();
    const elapsedMs = Math.max(0, nowMs - lastJoinedAtMs);

    if (elapsedMs > VISITED_BADGE_TTL_MS) {
        return null;
    }
    return (
        <Tooltip>
            <TooltipTrigger
                render={<span className={cn('inline-flex', className)} />}
            >
                <Badge
                    variant="outline"
                    className="border-border/70 text-muted-foreground h-4 rounded px-1 py-0 text-[10px] leading-none font-medium tabular-nums"
                >
                    {elapsedMs < HOUR_MS
                        ? t('side_panel.visited_minutes_ago', {
                              count: Math.max(
                                  1,
                                  Math.floor(elapsedMs / MINUTE_MS)
                              )
                          })
                        : t('side_panel.visited_hours_ago', {
                              count: Math.floor(elapsedMs / HOUR_MS)
                          })}
                </Badge>
            </TooltipTrigger>
            <TooltipContent>
                {t('side_panel.visited_at', {
                    time: formatDateFilter(lastJoinedAtMs, 'long')
                })}
            </TooltipContent>
        </Tooltip>
    );
}

export function InstanceVisitedBadge({
    location,
    className
}: {
    location: string;
    className?: string;
}) {
    const key = instanceLocationKey(location);
    const isCurrent = useRuntimeStore(
        (state) =>
            Boolean(key) &&
            instanceLocationKey(state.gameState.currentLocation) === key
    );
    const lastJoinedAtMs = useInstanceJoinHistoryStore((state) =>
        key ? state.lastJoinedAtByLocation[key] || 0 : 0
    );

    if (isCurrent || !lastJoinedAtMs) {
        return null;
    }
    return (
        <VisitedAgoBadge
            lastJoinedAtMs={lastJoinedAtMs}
            className={className}
        />
    );
}
