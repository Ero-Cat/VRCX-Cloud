import { ChevronsUpDownIcon } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
    FriendMultiSelectList,
    type FriendMultiSelectOption
} from '@/components/search/FriendMultiSelectList';
import type { FriendRosterById } from '@/domain/friends/types';
import { Button } from '@/ui/shadcn/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/ui/shadcn/popover';

function buildFriendPickerOptions(
    orderedFriendIds: string[],
    friendsById: FriendRosterById
): FriendMultiSelectOption[] {
    return orderedFriendIds.map((friendId) => {
        const user = friendsById[friendId] ?? null;
        const label = user?.displayName || friendId;
        return { value: friendId, label, search: `${label} ${friendId}`, user };
    });
}

function formatSelectedFriendLabels(labels: string[], emptyLabel: string) {
    if (!labels.length) {
        return emptyLabel;
    }

    const visibleLabels = labels.slice(0, 2).join(', ');
    if (labels.length <= 2) {
        return visibleLabels;
    }

    return `${visibleLabels} +${labels.length - 2}`;
}

export function FriendMultiSelect({
    disabled,
    friendsById,
    onChange,
    orderedFriendIds,
    values
}: {
    disabled?: boolean;
    friendsById: FriendRosterById;
    onChange: (next: string[]) => void;
    orderedFriendIds: string[];
    values: string[];
}) {
    const { t } = useTranslation();
    const [open, setOpen] = useState(false);
    const options = useMemo(
        () => buildFriendPickerOptions(orderedFriendIds, friendsById),
        [friendsById, orderedFriendIds]
    );
    const selectedLabels = values.map(
        (friendId) => friendsById[friendId]?.displayName || friendId
    );
    const triggerLabel = formatSelectedFriendLabels(
        selectedLabels,
        t('common.affinity.friend')
    );

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
                render={
                    <Button
                        type="button"
                        variant="outline"
                        className="w-full justify-between font-normal"
                        disabled={disabled}
                        aria-label={t('common.affinity.friend')}
                    >
                        <span className="truncate">{triggerLabel}</span>
                        <ChevronsUpDownIcon className="text-muted-foreground size-4" />
                    </Button>
                }
            />
            <PopoverContent align="start" className="w-96 p-2">
                <FriendMultiSelectList
                    autoFocus
                    options={options}
                    values={values}
                    onChange={onChange}
                    placeholder={t('view.friend_list.search_placeholder')}
                    emptyContent={t('empty_state.search_no_results')}
                />
            </PopoverContent>
        </Popover>
    );
}
