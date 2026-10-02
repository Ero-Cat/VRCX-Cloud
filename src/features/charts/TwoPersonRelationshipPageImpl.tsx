import {
    ArrowLeftRightIcon,
    ClockIcon,
    CrownIcon,
    HashIcon,
    InfoIcon,
    RefreshCcwIcon,
    UsersIcon
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSearchParams } from 'react-router';

import { displayNameForUser } from '@/components/dialogs/inviteDialogModel';
import { PageScaffold } from '@/components/layout/PageScaffold';
import { Location } from '@/components/Location';
import { FriendMultiSelectList } from '@/components/search/FriendMultiSelectList';
import type { FriendMultiSelectOption } from '@/components/search/FriendMultiSelectList';
import {
    buildSharedInstances,
    formatDuration,
    type SharedInstanceItem
} from '@/features/charts/two-person/twoPersonModel';
import { commands } from '@/platform/native/bindings';
import { toast } from '@/services/toastService';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Button } from '@/ui/shadcn/button';
import {
    HoverCard,
    HoverCardContent,
    HoverCardTrigger
} from '@/ui/shadcn/hover-card';
import { Spinner } from '@/ui/shadcn/spinner';
import { Switch } from '@/ui/shadcn/switch';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

const INITIATOR_LABEL_KEY = {
    mutual: 'view.charts.two_person_relationship.initiator_mutual',
    leftPlayer: 'view.charts.two_person_relationship.initiator_leftPlayer',
    rightPlayer: 'view.charts.two_person_relationship.initiator_rightPlayer',
    unknown: 'view.charts.two_person_relationship.initiator_unknown'
} as const;

function InitiatorBadge({
    initiator
}: {
    initiator: SharedInstanceItem['initiator'];
}) {
    const { t } = useTranslation();
    const base =
        'shrink-0 rounded px-1.5 py-0.5 text-[10px] leading-none font-medium';
    if (initiator === 'mutual') {
        return (
            <span
                className={`${base} bg-blue-500/15 text-blue-600 dark:text-blue-400`}
            >
                {t(INITIATOR_LABEL_KEY.mutual)}
            </span>
        );
    }
    if (initiator === 'unknown') {
        return (
            <span
                className={`${base} border border-zinc-200 bg-zinc-500/15 text-zinc-600 dark:border-zinc-800 dark:text-zinc-400`}
            >
                {t(INITIATOR_LABEL_KEY.unknown)}
            </span>
        );
    }
    return (
        <span
            className={`${base} text-orange-600 dark:text-orange-400`}
            style={
                initiator === 'leftPlayer'
                    ? {
                          background:
                              'linear-gradient(to right, rgb(249 115 22 / 0.3), rgb(249 115 22 / 0))'
                      }
                    : {
                          background:
                              'linear-gradient(to left, rgb(249 115 22 / 0.3), rgb(249 115 22 / 0))'
                      }
            }
        >
            {t(INITIATOR_LABEL_KEY[initiator])}
        </span>
    );
}

export function TwoPersonRelationshipPage() {
    const { t } = useTranslation();
    const [searchParams, setSearchParams] = useSearchParams();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentUser = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const [showSelfPresence, setShowSelfPresence] = useState(false);
    const [items, setItems] = useState<SharedInstanceItem[] | null>(null);
    const [loading, setLoading] = useState(false);
    const requestIdRef = useRef(0);

    const userIdA = searchParams.get('a') ?? '';
    const userIdB = searchParams.get('b') ?? '';

    const friendOptions = useMemo<FriendMultiSelectOption[]>(() => {
        const options: FriendMultiSelectOption[] = [];
        for (const friendId of Object.keys(friendsById)) {
            const label = displayNameForUser(
                friendId,
                friendsById,
                currentUser
            );
            options.push({
                value: friendId,
                label,
                search: `${label} ${friendId}`,
                user: friendsById[friendId] ?? null
            });
        }
        options.sort((left, right) => left.label.localeCompare(right.label));
        return options;
    }, [friendsById, currentUser]);

    const optionsA = useMemo(
        () => friendOptions.filter((option) => option.value !== userIdB),
        [friendOptions, userIdB]
    );
    const optionsB = useMemo(
        () => friendOptions.filter((option) => option.value !== userIdA),
        [friendOptions, userIdA]
    );

    const resolveName = useCallback(
        (userId: string) =>
            displayNameForUser(userId, friendsById, currentUser),
        [friendsById, currentUser]
    );

    const loadData = useCallback(async () => {
        if (!currentUserId || !userIdA || !userIdB || userIdA === userIdB) {
            setItems(null);
            return;
        }
        const requestId = ++requestIdRef.current;
        setLoading(true);
        try {
            const output = await commands.appTwoPersonRelationshipQuery({
                ownerUserId: currentUserId,
                userIdA,
                userIdB
            });
            if (requestId !== requestIdRef.current) {
                return;
            }
            setItems(buildSharedInstances(output, resolveName, false));
        } catch {
            if (requestId !== requestIdRef.current) {
                return;
            }
            setItems([]);
            toast.add({
                type: 'error',
                title: t('view.charts.two_person_relationship.no_data')
            });
        } finally {
            if (requestId === requestIdRef.current) {
                setLoading(false);
            }
        }
    }, [currentUserId, userIdA, userIdB, resolveName, t]);

    useEffect(() => {
        void loadData();
    }, [loadData]);

    function updateParam(key: string, value: string) {
        const next = new URLSearchParams(searchParams);
        if (value) {
            next.set(key, value);
        } else {
            next.delete(key);
        }
        setSearchParams(next, { replace: true });
    }

    function handleSelectA(values: string[]) {
        updateParam('a', values[values.length - 1] ?? '');
    }

    function handleSelectB(values: string[]) {
        updateParam('b', values[values.length - 1] ?? '');
    }

    function swapFriends() {
        const next = new URLSearchParams(searchParams);
        next.set('a', userIdB);
        next.set('b', userIdA);
        setSearchParams(next, { replace: true });
    }

    const totalCoexistenceMs = useMemo(
        () =>
            (items ?? []).reduce(
                (total, item) => total + item.coexistenceTimeMs,
                0
            ),
        [items]
    );

    return (
        <PageScaffold id="chart" className="overflow-y-auto p-0">
            <div className="pt-4">
                <div className="mt-0 flex flex-col gap-2 px-4">
                    <div className="flex items-center gap-2">
                        <span className="shrink-0">
                            {t('view.charts.two_person_relationship.header')}
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
                                        'view.charts.two_person_relationship.tips.description'
                                    )}
                                </div>
                            </HoverCardContent>
                        </HoverCard>
                    </div>

                    <div className="flex flex-wrap items-center gap-2">
                        <div className="min-w-52 flex-1">
                            <FriendMultiSelectList
                                options={optionsA}
                                values={userIdA ? [userIdA] : []}
                                onChange={handleSelectA}
                                placeholder={t(
                                    'view.charts.two_person_relationship.select_friend_a'
                                )}
                                emptyContent={t(
                                    'view.charts.two_person_relationship.no_data'
                                )}
                            />
                        </div>
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button
                                        type="button"
                                        className="shrink-0 rounded-full"
                                        size="icon"
                                        variant="ghost"
                                        disabled={!userIdA && !userIdB}
                                        onClick={swapFriends}
                                    />
                                }
                            >
                                <ArrowLeftRightIcon className="size-4" />
                            </TooltipTrigger>
                            <TooltipContent side="top">
                                {t(
                                    'view.charts.two_person_relationship.swap_friends'
                                )}
                            </TooltipContent>
                        </Tooltip>
                        <div className="min-w-52 flex-1">
                            <FriendMultiSelectList
                                options={optionsB}
                                values={userIdB ? [userIdB] : []}
                                onChange={handleSelectB}
                                placeholder={t(
                                    'view.charts.two_person_relationship.select_friend_b'
                                )}
                                emptyContent={t(
                                    'view.charts.two_person_relationship.no_data'
                                )}
                            />
                        </div>
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="icon"
                                        className="shrink-0 rounded-full"
                                        disabled={
                                            !userIdA || !userIdB || loading
                                        }
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
                                {t('common.actions.refresh')}
                            </TooltipContent>
                        </Tooltip>
                        <div className="ml-auto flex shrink-0 items-center gap-2 px-0.5">
                            <span className="shrink-0 text-sm">
                                {t(
                                    'view.charts.two_person_relationship.show_self_presence'
                                )}
                            </span>
                            <Switch
                                checked={showSelfPresence}
                                onCheckedChange={setShowSelfPresence}
                            />
                        </div>
                    </div>
                </div>

                {loading && items === null ? (
                    <div className="mt-[100px] flex items-center justify-center">
                        <RefreshCcwIcon className="text-muted-foreground size-6 animate-spin" />
                    </div>
                ) : !userIdA || !userIdB ? (
                    <div className="text-muted-foreground mt-[100px] flex flex-col items-center justify-center gap-2">
                        <UsersIcon className="size-12 opacity-20" />
                        <p>
                            {t(
                                'view.charts.two_person_relationship.no_friend_selected'
                            )}
                        </p>
                    </div>
                ) : items !== null && items.length === 0 ? (
                    <div className="text-muted-foreground mt-[100px] flex flex-col items-center justify-center gap-2">
                        <UsersIcon className="size-12 opacity-20" />
                        <p>
                            {t('view.charts.two_person_relationship.no_data')}
                        </p>
                    </div>
                ) : items ? (
                    <>
                        <div className="mx-auto mt-3 flex max-w-[900px] items-center gap-3">
                            <div className="flex items-center gap-2 rounded-lg border px-3 py-2">
                                <ClockIcon className="text-muted-foreground size-3.5" />
                                <span className="text-sm font-medium tabular-nums">
                                    {formatDuration(totalCoexistenceMs)}
                                </span>
                                <span className="text-muted-foreground text-xs">
                                    {t(
                                        'view.charts.two_person_relationship.total_coexistence_time'
                                    )}
                                </span>
                            </div>
                            <div className="flex items-center gap-2 rounded-lg border px-3 py-2">
                                <HashIcon className="text-muted-foreground size-3.5" />
                                <span className="text-sm font-medium tabular-nums">
                                    {items.length}
                                </span>
                                <span className="text-muted-foreground text-xs">
                                    {t(
                                        'view.charts.two_person_relationship.instance_count'
                                    )}
                                </span>
                            </div>
                        </div>

                        <div className="mx-auto mt-3 flex max-w-[900px] flex-col gap-3 pb-8">
                            {items.map((item) => (
                                <div
                                    key={item.location}
                                    className="hover:bg-accent group flex w-full items-center gap-3 rounded-lg border p-3 text-left transition-all"
                                >
                                    <div className="text-muted-foreground w-36 shrink-0 text-xs tabular-nums">
                                        {item.formattedDate}
                                    </div>
                                    <div className="min-w-0 flex-1">
                                        <Location location={item.location} />
                                        <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5">
                                            {item.instanceCreatorName ? (
                                                <Tooltip>
                                                    <TooltipTrigger
                                                        render={
                                                            <span className="text-muted-foreground flex items-center gap-1 text-xs" />
                                                        }
                                                    >
                                                        <CrownIcon className="size-3 shrink-0" />
                                                        <span className="max-w-[120px] truncate">
                                                            {
                                                                item.instanceCreatorName
                                                            }
                                                        </span>
                                                    </TooltipTrigger>
                                                    <TooltipContent side="top">
                                                        {t(
                                                            'view.charts.two_person_relationship.instance_creator'
                                                        )}
                                                    </TooltipContent>
                                                </Tooltip>
                                            ) : null}
                                            {item.maxPlayerCount !== null ? (
                                                <Tooltip>
                                                    <TooltipTrigger
                                                        render={
                                                            <span className="text-muted-foreground flex items-center gap-1 text-xs" />
                                                        }
                                                    >
                                                        <UsersIcon className="size-3 shrink-0" />
                                                        <span className="tabular-nums">
                                                            {
                                                                item.maxPlayerCount
                                                            }
                                                        </span>
                                                    </TooltipTrigger>
                                                    <TooltipContent side="top">
                                                        {t(
                                                            'view.charts.two_person_relationship.max_player_count'
                                                        )}
                                                    </TooltipContent>
                                                </Tooltip>
                                            ) : null}
                                        </div>
                                    </div>
                                    <div className="flex shrink-0 flex-col items-end gap-1.5">
                                        <div className="text-muted-foreground mr-1 flex items-center gap-1.5 text-xs">
                                            {item.joinLeavesCount > 1 ? (
                                                <span className="bg-muted rounded px-1.5 py-0.5 text-[10px] leading-none tabular-nums opacity-80">
                                                    {t(
                                                        'view.charts.two_person_relationship.meet_count',
                                                        {
                                                            count: item.joinLeavesCount
                                                        }
                                                    )}
                                                </span>
                                            ) : null}
                                            <ClockIcon className="size-3 shrink-0" />
                                            <span className="font-medium tabular-nums">
                                                {formatDuration(
                                                    item.coexistenceTimeMs
                                                )}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            {showSelfPresence ? (
                                                <span
                                                    className={`shrink-0 rounded px-1.5 py-0.5 text-[10px] leading-none font-medium ${
                                                        item.selfPresent
                                                            ? 'bg-green-500/15 text-green-600 dark:text-green-400'
                                                            : 'bg-red-500/15 text-red-600 dark:text-red-400'
                                                    }`}
                                                >
                                                    {item.selfPresent
                                                        ? t(
                                                              'view.charts.two_person_relationship.self_present'
                                                          )
                                                        : t(
                                                              'view.charts.two_person_relationship.self_not_present'
                                                          )}
                                                </span>
                                            ) : null}
                                            <InitiatorBadge
                                                initiator={item.initiator}
                                            />
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </>
                ) : null}
            </div>
        </PageScaffold>
    );
}
