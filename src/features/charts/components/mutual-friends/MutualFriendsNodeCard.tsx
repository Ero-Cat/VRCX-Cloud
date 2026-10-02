import {
    EyeOffIcon,
    LinkIcon,
    RefreshCcwIcon,
    ScanSearchIcon,
    UserIcon,
    XIcon
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FadeInImage } from '@/components/media/FadeInImage';
import type { FriendRecord } from '@/domain/friends/types';
import { formatDateFilter } from '@/lib/dateTime';
import type {
    MutualFriendCommunity,
    MutualFriendNode
} from '@/lib/mutual-friends/mutualFriendsTypes';
import { userImage } from '@/services/entityMediaService';
import { Button } from '@/ui/shadcn/button';
import { Input } from '@/ui/shadcn/input';
import { Spinner } from '@/ui/shadcn/spinner';

import { MutualFriendsSurface } from './MutualFriendsSurface';

type LinkableNodeOption = { value: string; label: string };

export function MutualFriendsNodeCard({
    community,
    isRefreshing,
    linkableNodeOptions = [],
    node,
    onAddManualLink,
    onClose,
    onFocusCommunity,
    onHide,
    onOpenProfile,
    onRefresh,
    user
}: {
    community: MutualFriendCommunity | null;
    isRefreshing: boolean;
    linkableNodeOptions?: LinkableNodeOption[];
    node: MutualFriendNode;
    onAddManualLink: (targetId: string) => void;
    onClose: () => void;
    onFocusCommunity: () => void;
    onHide: () => void;
    onOpenProfile: () => void;
    onRefresh: () => void;
    user: FriendRecord | null;
}) {
    const { t } = useTranslation();
    const imageUrl = user ? userImage(user, 128) : '';
    const [linkPickerOpen, setLinkPickerOpen] = useState(false);
    const [linkQuery, setLinkQuery] = useState('');
    const filteredLinkOptions = useMemo(() => {
        const query = linkQuery.trim().toLowerCase();
        if (!query) {
            return linkableNodeOptions.slice(0, 12);
        }
        return linkableNodeOptions
            .filter(
                (option) =>
                    option.label.toLowerCase().includes(query) ||
                    option.value.toLowerCase().includes(query)
            )
            .slice(0, 12);
    }, [linkableNodeOptions, linkQuery]);

    return (
        <MutualFriendsSurface className="animate-in fade-in-0 slide-in-from-bottom-2 pointer-events-auto absolute right-3 bottom-3 z-10 w-72 p-3 duration-200 ease-out">
            <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                className="absolute top-2 right-2 size-6"
                aria-label={t('common.actions.close')}
                onClick={onClose}
            >
                <XIcon className="size-3.5" />
            </Button>

            <div className="flex items-center gap-3 pr-6">
                <span className="bg-muted flex size-11 shrink-0 items-center justify-center overflow-hidden rounded-full border">
                    {imageUrl ? (
                        <FadeInImage
                            src={imageUrl}
                            alt=""
                            loading="lazy"
                            className="size-full object-cover"
                            fallback={
                                <UserIcon className="text-muted-foreground size-4" />
                            }
                        />
                    ) : (
                        <UserIcon className="text-muted-foreground size-4" />
                    )}
                </span>
                <div className="min-w-0 flex-1">
                    <p className="truncate text-sm leading-5 font-medium">
                        {node.label}
                    </p>
                    <p className="text-muted-foreground truncate text-xs">
                        {node.degree} {t('view.charts.label.connections')}
                    </p>
                </div>
            </div>

            {community ? (
                <button
                    type="button"
                    onClick={onFocusCommunity}
                    className="mt-3 flex w-full items-center gap-2 rounded-md px-1.5 py-1 text-left transition-colors duration-150 ease-out hover:bg-(--state-hover-surface) active:translate-y-px"
                >
                    <span
                        className="size-2.5 shrink-0 rounded-full"
                        style={{ backgroundColor: community.color }}
                    />
                    <span className="min-w-0 flex-1 truncate text-xs">
                        {community.label}
                    </span>
                    <ScanSearchIcon className="text-muted-foreground size-3.5 shrink-0" />
                </button>
            ) : null}

            <p className="text-muted-foreground mt-2 px-1.5 text-xs">
                {node.optedOut
                    ? t('view.charts.mutual_friend.label.mutuals_unavailable')
                    : node.lastFetchedAt
                      ? `${t('view.charts.mutual_friend.context_menu.last_fetched')}: ${formatDateFilter(node.lastFetchedAt, 'long')}`
                      : t('view.charts.mutual_friend.label.never_fetched')}
            </p>

            <div className="mt-3 flex items-center gap-1.5">
                <Button
                    type="button"
                    size="sm"
                    className="flex-1"
                    onClick={onOpenProfile}
                >
                    {t('view.charts.mutual_friend.actions.open_profile')}
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    aria-label={t(
                        'view.charts.mutual_friend.actions.link_to_node'
                    )}
                    onClick={() => setLinkPickerOpen((open) => !open)}
                >
                    <LinkIcon />
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    aria-label={t('common.actions.refresh')}
                    disabled={isRefreshing}
                    onClick={onRefresh}
                >
                    {isRefreshing ? <Spinner /> : <RefreshCcwIcon />}
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    aria-label={t(
                        'view.charts.mutual_friend.actions.hide_from_graph'
                    )}
                    onClick={onHide}
                >
                    <EyeOffIcon />
                </Button>
            </div>

            {linkPickerOpen ? (
                <div className="mt-2 flex flex-col gap-1.5">
                    <Input
                        value={linkQuery}
                        onChange={(event) => setLinkQuery(event.target.value)}
                        placeholder={t(
                            'view.charts.mutual_friend.actions.link_to_node'
                        )}
                        className="h-8 text-xs"
                    />
                    <div className="flex max-h-40 flex-col gap-0.5 overflow-y-auto">
                        {filteredLinkOptions.map((option) => (
                            <Button
                                key={option.value}
                                type="button"
                                variant="ghost"
                                size="sm"
                                className="justify-start text-xs"
                                onClick={() => {
                                    onAddManualLink(option.value);
                                    setLinkPickerOpen(false);
                                    setLinkQuery('');
                                }}
                            >
                                <span className="truncate">{option.label}</span>
                            </Button>
                        ))}
                        {filteredLinkOptions.length === 0 ? (
                            <span className="text-muted-foreground px-2 py-1 text-xs">
                                {t('dialog.mutual_instances.no_results')}
                            </span>
                        ) : null}
                    </div>
                </div>
            ) : null}
        </MutualFriendsSurface>
    );
}
