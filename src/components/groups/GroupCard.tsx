import { UsersIcon, UsersRoundIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import { cn } from '@/lib/utils';
import { convertFileUrlToImageUrl } from '@/services/entityMediaService';
import { Avatar, AvatarFallback, AvatarImage } from '@/ui/shadcn/avatar';
import { Badge } from '@/ui/shadcn/badge';
import { Button } from '@/ui/shadcn/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

export type GroupCardData = {
    id: string;
    description?: string | null;
    discriminator?: string | null;
    iconUrl?: string | null;
    memberCount?: number | null;
    name?: string | null;
    shortCode?: string | null;
};

export function GroupCard({
    actions,
    group,
    onClick
}: {
    actions?: ReactNode;
    group: GroupCardData;
    onClick: () => void;
}) {
    const imageUrl = convertFileUrlToImageUrl(group.iconUrl);
    const groupCode =
        group.shortCode && group.discriminator
            ? `${group.shortCode}.${group.discriminator}`
            : group.shortCode || group.discriminator || null;

    return (
        <div className="relative min-w-0">
            <Button
                type="button"
                variant="outline"
                className={cn(
                    'h-auto w-full min-w-0 items-start justify-start gap-3 overflow-hidden p-3 text-left font-normal whitespace-normal',
                    actions && 'pr-12'
                )}
                onClick={onClick}
            >
                <Avatar className="size-14 rounded-lg after:rounded-lg">
                    {imageUrl ? (
                        <AvatarImage
                            src={imageUrl}
                            alt={group.name || 'Group'}
                            loading="lazy"
                            className="rounded-lg"
                        />
                    ) : null}
                    <AvatarFallback className="rounded-lg [&>svg]:size-5">
                        <UsersRoundIcon aria-hidden="true" />
                    </AvatarFallback>
                </Avatar>
                <span className="flex min-w-0 flex-1 flex-col gap-2 overflow-hidden">
                    <span className="flex max-w-full min-w-0 items-center gap-1.5">
                        <span className="min-w-0 truncate text-sm font-semibold">
                            {group.name || ''}
                        </span>
                        <Badge
                            variant="secondary"
                            className="shrink-0 rounded-sm px-1.5 tabular-nums"
                        >
                            <UsersIcon data-icon="inline-start" />
                            {group.memberCount ?? 0}
                        </Badge>
                    </span>
                    {groupCode ? (
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Badge
                                        variant="outline"
                                        className="max-w-full min-w-0 justify-start rounded-sm px-1.5 font-mono"
                                    >
                                        <span className="min-w-0 truncate">
                                            {groupCode}
                                        </span>
                                    </Badge>
                                }
                            />
                            <TooltipContent className="max-w-72 break-words">
                                {groupCode}
                            </TooltipContent>
                        </Tooltip>
                    ) : null}
                    {group.description ? (
                        <span className="text-muted-foreground line-clamp-2 text-xs leading-snug break-words">
                            {group.description}
                        </span>
                    ) : null}
                </span>
            </Button>
            {actions ? (
                <div className="absolute top-2 right-2 z-10">{actions}</div>
            ) : null}
        </div>
    );
}
