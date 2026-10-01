import { Combobox as ComboboxPrimitive } from '@base-ui/react/combobox';
import { CheckIcon, UserIcon } from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FadeInImage } from '@/components/media/FadeInImage';
import { UserPickerRow } from '@/components/search/UserPickerRow';
import { cn } from '@/lib/utils';
import { userImage } from '@/services/entityMediaService';
import { Badge } from '@/ui/shadcn/badge';
import {
    Combobox,
    ComboboxChip,
    ComboboxChips,
    ComboboxChipsInput,
    ComboboxValue
} from '@/ui/shadcn/combobox';
import { ScrollArea } from '@/ui/shadcn/scroll-area';

export type FriendMultiSelectOption = {
    badge?: string;
    label: string;
    search: string;
    user: Record<string, unknown> | null;
    value: string;
};

export function friendOptionMatches(
    option: Pick<FriendMultiSelectOption, 'search'>,
    query: string
) {
    const text = option.search.toLowerCase();
    return query
        .toLowerCase()
        .split(/\s+/)
        .filter(Boolean)
        .every((token) => text.includes(token));
}

function isSameOption(
    left: FriendMultiSelectOption,
    right: FriendMultiSelectOption
) {
    return left.value === right.value;
}

export function FriendMultiSelectList({
    autoFocus,
    disabled,
    emptyContent,
    limit = 100,
    listClassName,
    onChange,
    options,
    placeholder,
    values
}: {
    autoFocus?: boolean;
    disabled?: boolean;
    emptyContent: ReactNode;
    limit?: number;
    listClassName?: string;
    onChange: (next: string[]) => void;
    options: FriendMultiSelectOption[];
    placeholder: string;
    values: string[];
}) {
    const { t } = useTranslation();
    const [query, setQuery] = useState('');

    const items = useMemo(() => {
        const known = new Set(options.map((option) => option.value));
        const orphaned = values
            .filter((value) => value && !known.has(value))
            .map<FriendMultiSelectOption>((value) => ({
                value,
                label: value,
                search: value.toLowerCase(),
                user: null
            }));
        return orphaned.length ? [...options, ...orphaned] : options;
    }, [options, values]);

    const selectedOptions = useMemo(() => {
        const byValue = new Map(items.map((item) => [item.value, item]));
        return values.flatMap((value) => byValue.get(value) ?? []);
    }, [items, values]);

    const matched = useMemo(
        () => items.filter((item) => friendOptionMatches(item, query)),
        [items, query]
    );
    const filteredItems = useMemo(
        () => matched.slice(0, limit),
        [limit, matched]
    );
    const hiddenCount = matched.length - filteredItems.length;

    return (
        <Combobox
            inline
            open
            multiple
            disabled={disabled}
            items={items}
            filteredItems={filteredItems}
            value={selectedOptions}
            onValueChange={(next) => onChange(next.map((item) => item.value))}
            isItemEqualToValue={isSameOption}
            inputValue={query}
            onInputValueChange={setQuery}
        >
            <div className="flex flex-col gap-2">
                <ComboboxChips className="max-h-24 overflow-y-auto">
                    <ComboboxValue>
                        {(selected: FriendMultiSelectOption[]) => (
                            <>
                                {selected.map((item) => {
                                    const imageUrl = item.user
                                        ? userImage(item.user, 64)
                                        : '';
                                    return (
                                        <ComboboxChip
                                            key={item.value}
                                            aria-label={item.label}
                                            removeLabel={`${t('common.actions.remove')} ${item.label}`}
                                            className="max-w-48 gap-1.5 pl-1"
                                        >
                                            <span className="bg-muted flex size-3.5 shrink-0 items-center justify-center overflow-hidden rounded-full">
                                                {imageUrl ? (
                                                    <FadeInImage
                                                        src={imageUrl}
                                                        alt=""
                                                        loading="lazy"
                                                        className="size-full object-cover"
                                                    />
                                                ) : (
                                                    <UserIcon className="text-muted-foreground size-2.5" />
                                                )}
                                            </span>
                                            <span className="truncate">
                                                {item.label}
                                            </span>
                                        </ComboboxChip>
                                    );
                                })}
                                <ComboboxChipsInput
                                    autoFocus={autoFocus}
                                    placeholder={
                                        selected.length
                                            ? undefined
                                            : placeholder
                                    }
                                    aria-label={placeholder}
                                />
                            </>
                        )}
                    </ComboboxValue>
                </ComboboxChips>
                <ScrollArea
                    className={cn('h-72 rounded-md border', listClassName)}
                >
                    <ComboboxPrimitive.List className="flex flex-col gap-0.5 p-1 pr-2">
                        {(item: FriendMultiSelectOption) => (
                            <ComboboxPrimitive.Item
                                key={item.value}
                                value={item}
                                className="group/item data-highlighted:bg-muted flex cursor-pointer items-center gap-1 rounded-md pl-2 outline-none select-none data-disabled:pointer-events-none data-disabled:opacity-50"
                            >
                                <span className="border-input group-data-[selected]/item:border-primary group-data-[selected]/item:bg-primary group-data-[selected]/item:text-primary-foreground flex size-4 shrink-0 items-center justify-center rounded-[4px] border transition-[background-color,border-color,transform] duration-100 ease-out group-active/item:scale-95">
                                    <CheckIcon className="size-3.5 opacity-0 group-data-[selected]/item:opacity-100" />
                                </span>
                                <span className="flex min-w-0 flex-1 items-center">
                                    <UserPickerRow
                                        option={item}
                                        showSelection={false}
                                    />
                                    {item.badge ? (
                                        <Badge
                                            variant="outline"
                                            className="mr-1.5 max-w-32 shrink-0 truncate"
                                        >
                                            {item.badge}
                                        </Badge>
                                    ) : null}
                                </span>
                            </ComboboxPrimitive.Item>
                        )}
                    </ComboboxPrimitive.List>
                    <ComboboxPrimitive.Empty className="text-muted-foreground text-xs not-empty:p-3">
                        {emptyContent}
                    </ComboboxPrimitive.Empty>
                    <ComboboxPrimitive.Status className="text-muted-foreground text-xs not-empty:px-3 not-empty:pb-2">
                        {hiddenCount
                            ? t('common.picker.more_not_shown', {
                                  count: hiddenCount
                              })
                            : null}
                    </ComboboxPrimitive.Status>
                </ScrollArea>
            </div>
        </Combobox>
    );
}
