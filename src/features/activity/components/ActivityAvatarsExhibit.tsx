import { ShirtIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { FadeInImage } from '@/components/media/FadeInImage';
import type { AvatarUsageRow } from '@/platform/tauri/bindings';
import { openAvatarDialog } from '@/services/dialogService';

import { Exhibit } from './ActivityExhibit';
import { RankRow } from './ActivityRankRow';

function hours(milliseconds: number) {
    return (milliseconds / 3_600_000).toFixed(1);
}

function Thumb({ url, className }: { url: string; className: string }) {
    const placeholder = (
        <span
            className={`${className} flex items-center justify-center rounded-md bg-[var(--act-track)]`}
        >
            <ShirtIcon className="text-muted-foreground size-4" />
        </span>
    );
    if (!url) {
        return placeholder;
    }
    return (
        <FadeInImage
            src={url}
            alt=""
            loading="lazy"
            decoding="async"
            className={`${className} rounded-md object-cover`}
            fallback={placeholder}
        />
    );
}

export function ActivityAvatarsExhibit({ rows }: { rows: AvatarUsageRow[] }) {
    const { t } = useTranslation();
    const [lead, ...rest] = rows;

    if (!lead) {
        return null;
    }

    const hoursUnit = t('view.activity.unit.hours');
    const nameOf = (row: AvatarUsageRow) =>
        row.name || t('browse_history.unknown.avatar');

    return (
        <Exhibit
            label={t('view.activity.section.avatars')}
            caption={t('view.activity.avatars.all_time')}
            detailLabel={t('view.activity.worlds.open_ranking')}
            detail={
                <div className="flex flex-col">
                    {rest.map((row, index) => (
                        <RankRow
                            key={row.avatarId}
                            index={index + 1}
                            onClick={() =>
                                openAvatarDialog({ avatarId: row.avatarId })
                            }
                            leading={
                                <Thumb
                                    url={row.thumbnailImageUrl}
                                    className="size-9 shrink-0"
                                />
                            }
                            title={nameOf(row)}
                            primary={`${hours(row.timeSpent)}${hoursUnit}`}
                        />
                    ))}
                </div>
            }
        >
            <button
                type="button"
                onClick={() => openAvatarDialog({ avatarId: lead.avatarId })}
                className="flex w-full items-center gap-4 text-left transition-opacity duration-100 ease-out hover:opacity-85 active:opacity-70"
            >
                <Thumb
                    url={lead.thumbnailImageUrl || lead.imageUrl}
                    className="block size-16 shrink-0"
                />
                <span className="min-w-0 flex-1">
                    <span className="text-foreground block truncate text-xl font-semibold tracking-[-0.015em]">
                        {nameOf(lead)}
                    </span>
                    <span className="text-muted-foreground mt-1.5 block text-xs tabular-nums">
                        {hours(lead.timeSpent)}
                        {hoursUnit}
                    </span>
                </span>
            </button>
        </Exhibit>
    );
}
