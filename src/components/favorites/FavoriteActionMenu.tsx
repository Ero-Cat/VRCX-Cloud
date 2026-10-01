import { PlusIcon, StarIcon } from 'lucide-react';
import { useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useLocalWorldFavorites } from '@/components/favorites/useLocalWorldFavorites';
import type {
    FavoriteGroup as FavoriteStoreGroup,
    FavoriteGroupMap,
    FavoriteKind,
    FavoriteRecord,
    FavoriteStore,
    RemoteFavoriteKind
} from '@/domain/favorites/types';
import type { VrchatFavoriteType } from '@/platform/tauri/bindings';
import favoritePersistenceRepository from '@/repositories/favoritePersistenceRepository';
import vrchatFavoriteRepository from '@/repositories/vrchatFavoriteRepository';
import { persistAvatarDetails } from '@/services/favoriteAvatarCacheService';
import { persistWorldDetails } from '@/services/favoriteWorldCacheService';
import { toast } from '@/services/toastService';
import { isRecord } from '@/shared/utils/record';
import { useFavoriteStore } from '@/state/favoriteStore';
import { useModalStore } from '@/state/modalStore';
import { Button } from '@/ui/shadcn/button';
import {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger
} from '@/ui/shadcn/dropdown-menu';
import { Spinner } from '@/ui/shadcn/spinner';

const EMPTY_GROUPS: FavoriteStoreGroup[] = [];
const EMPTY_LOCAL_GROUPS: string[] = [];
const EMPTY_FAVORITES: FavoriteGroupMap = {};

type FavoriteActionMenuProps = {
    kind: FavoriteKind;
    entityId: string;
    entity?: unknown;
    label?: string;
    iconOnly?: boolean;
};

function normalizeEntityId(value: unknown) {
    return typeof value === 'string'
        ? value.trim()
        : String(value ?? '').trim();
}

function resolveGroups(kind: FavoriteKind, state: FavoriteStore) {
    if (kind === 'friend') {
        return state.favoriteFriendGroups;
    }
    if (kind === 'avatar') {
        return state.favoriteAvatarGroups;
    }
    if (kind === 'world') {
        return state.favoriteWorldGroups;
    }
    return EMPTY_GROUPS;
}

function resolveLocalGroups(kind: FavoriteKind, state: FavoriteStore) {
    if (kind === 'friend') {
        return state.localFriendFavoriteGroups;
    }
    if (kind === 'avatar') {
        return state.localAvatarFavoriteGroups;
    }
    return EMPTY_LOCAL_GROUPS;
}

function resolveLocalFavorites(kind: FavoriteKind, state: FavoriteStore) {
    if (kind === 'friend') {
        return state.localFriendFavorites || {};
    }
    if (kind === 'avatar') {
        return state.localAvatarFavorites || {};
    }
    return EMPTY_FAVORITES;
}

function formatGroupLabel(group: FavoriteStoreGroup) {
    const count = Number(group.count) || 0;
    const capacity = Number(group.capacity) || 0;
    const suffix =
        capacity > 0 ? ` (${count}/${capacity})` : count ? ` (${count})` : '';
    return `${String(group.displayName || group.name || group.key)}${suffix}`;
}

export function resolveFavoriteAddType(
    group: FavoriteStoreGroup,
    fallbackKind: FavoriteKind
): VrchatFavoriteType {
    const type: RemoteFavoriteKind | undefined = group.type;
    if (isVrchatFavoriteType(type)) {
        return type;
    }
    return fallbackKind;
}

function isVrchatFavoriteType(
    type: RemoteFavoriteKind | undefined
): type is VrchatFavoriteType {
    return (
        type === 'friend' ||
        type === 'avatar' ||
        type === 'world' ||
        type === 'vrcPlusWorld'
    );
}

function remoteFavoriteGroupKey(
    remoteFavorite: FavoriteRecord | null | undefined
) {
    return normalizeEntityId(remoteFavorite?.$groupKey);
}

function remoteFavoriteTag(remoteFavorite: FavoriteRecord | null | undefined) {
    return Array.isArray(remoteFavorite?.tags)
        ? normalizeEntityId(remoteFavorite.tags[0])
        : '';
}

function resolveRemoteFavoriteGroup(
    remoteFavorite: FavoriteRecord | null | undefined,
    groups: readonly FavoriteStoreGroup[] | null | undefined
) {
    const type = normalizeEntityId(remoteFavorite?.type);
    const tag = remoteFavoriteTag(remoteFavorite);
    const candidates = new Set(
        [
            remoteFavoriteGroupKey(remoteFavorite),
            tag.includes(':') ? tag : '',
            type && tag ? `${type}:${tag}` : ''
        ].filter(Boolean)
    );
    return (Array.isArray(groups) ? groups : EMPTY_GROUPS).find((item) =>
        candidates.has(normalizeEntityId(item?.key))
    );
}

function hasLocalFavorite(
    localFavorites: FavoriteGroupMap,
    groupName: string,
    entityId: string
) {
    return (
        Array.isArray(localFavorites?.[groupName]) &&
        localFavorites[groupName].some(
            (value) => normalizeEntityId(value) === entityId
        )
    );
}

function localGroupLabel(localFavorites: FavoriteGroupMap, groupName: string) {
    const count = Array.isArray(localFavorites?.[groupName])
        ? localFavorites[groupName].length
        : 0;
    return `${groupName} (${count})`;
}

export function FavoriteActionMenu({
    kind,
    entityId,
    entity = null,
    label = '',
    iconOnly = false
}: FavoriteActionMenuProps) {
    const { t } = useTranslation();

    const normalizedEntityId = entityId.trim();
    const prompt = useModalStore((state) => state.prompt);
    const groups = useFavoriteStore((state) => resolveGroups(kind, state));
    const localWorldFavorites = useLocalWorldFavorites(
        kind === 'world' && Boolean(normalizedEntityId)
    );
    const storedLocalGroups = useFavoriteStore((state) =>
        resolveLocalGroups(kind, state)
    );
    const storedLocalFavorites = useFavoriteStore((state) =>
        resolveLocalFavorites(kind, state)
    );
    const localFavorites =
        kind === 'world'
            ? localWorldFavorites.favoritesByGroup
            : storedLocalFavorites;
    const localGroups = useMemo(() => {
        if (kind === 'world') {
            return localWorldFavorites.groupNames;
        }
        if (storedLocalGroups.length) {
            return storedLocalGroups;
        }
        return Object.keys(localFavorites);
    }, [
        kind,
        localFavorites,
        localWorldFavorites.groupNames,
        storedLocalGroups
    ]);
    const localFavoriteActive = useMemo(
        () =>
            localGroups.some((groupName) =>
                hasLocalFavorite(localFavorites, groupName, normalizedEntityId)
            ),
        [localFavorites, localGroups, normalizedEntityId]
    );
    const remoteFavorite = useFavoriteStore(
        (state) => state.remoteFavoritesByObjectId[normalizedEntityId] || null
    );
    const remoteFavoriteGroup = useMemo(
        () => resolveRemoteFavoriteGroup(remoteFavorite, groups),
        [groups, remoteFavorite]
    );
    const [actionStatus, setActionStatus] = useState('idle');
    const actionStatusRef = useRef('idle');

    async function setRemoteFavoriteGroup(group: FavoriteStoreGroup | null) {
        if (!normalizedEntityId || actionStatusRef.current !== 'idle') {
            return;
        }

        actionStatusRef.current = 'favorite';
        setActionStatus('favorite');
        try {
            if (remoteFavorite) {
                await vrchatFavoriteRepository.deleteFavorite({
                    objectId: normalizedEntityId
                });
            }
            if (group) {
                await vrchatFavoriteRepository.addFavorite({
                    type: resolveFavoriteAddType(group, kind),
                    favoriteId: normalizedEntityId,
                    tags: group.name
                });
                if (kind === 'world' && isRecord(entity)) {
                    persistWorldDetails(entity, normalizedEntityId);
                } else if (kind === 'avatar' && isRecord(entity)) {
                    persistAvatarDetails(entity, normalizedEntityId);
                }
            }
            toast.add({
                type: 'success',
                title: group
                    ? t('view.favorite.label.favorite_added')
                    : t('view.favorite.success.favorite_removed')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : group
                          ? t(
                                'component.favorite_action_menu.toast.failed_to_add_favorite'
                            )
                          : t(
                                'component.favorite_action_menu.toast.failed_to_remove_favorite'
                            )
            });
        } finally {
            actionStatusRef.current = 'idle';
            setActionStatus('idle');
        }
    }

    async function addLocalFavoriteToGroup(groupName: string) {
        if (!normalizedEntityId || actionStatusRef.current !== 'idle') {
            return;
        }

        actionStatusRef.current = 'local-favorite';
        setActionStatus('local-favorite');
        try {
            if (kind === 'world' && isRecord(entity)) {
                persistWorldDetails(entity, normalizedEntityId);
            } else if (kind === 'avatar' && isRecord(entity)) {
                persistAvatarDetails(entity, normalizedEntityId);
            }
            await favoritePersistenceRepository.addLocalFavorite({
                kind,
                entityId: normalizedEntityId,
                groupName
            });
            toast.add({
                type: 'success',
                title: t('view.favorite.label.local_favorite_added')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'component.favorite_action_menu.toast.failed_to_add_local_favorite'
                          )
            });
        } finally {
            actionStatusRef.current = 'idle';
            setActionStatus('idle');
        }
    }

    async function removeLocalFavoriteFromGroup(groupName: string) {
        if (!normalizedEntityId || actionStatusRef.current !== 'idle') {
            return;
        }

        actionStatusRef.current = 'local-favorite';
        setActionStatus('local-favorite');
        try {
            await favoritePersistenceRepository.removeLocalFavorite({
                kind,
                entityId: normalizedEntityId,
                groupName
            });
            toast.add({
                type: 'success',
                title: t('view.favorite.success.local_favorite_removed')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'component.favorite_action_menu.toast.failed_to_remove_local_favorite'
                          )
            });
        } finally {
            actionStatusRef.current = 'idle';
            setActionStatus('idle');
        }
    }

    async function createLocalFavoriteGroupAndAdd() {
        if (!normalizedEntityId || actionStatusRef.current !== 'idle') {
            return;
        }
        const result = await prompt({
            title: t('view.favorite.worlds.new_group'),
            description: t(
                'view.favorites.modal.enter_the_new_local_group_name'
            ),
            inputValue: '',
            pattern: /\S+/,
            confirmText: t('common.actions.confirm'),
            cancelText: t('common.actions.cancel')
        });
        if (!result.ok || typeof result.value !== 'string') {
            return;
        }
        const groupName = result.value.trim();
        if (!groupName) {
            return;
        }
        if (localGroups.includes(groupName)) {
            toast.add({
                type: 'error',
                title: t(
                    'view.favorites.dynamic.local_group_value_already_exists',
                    {
                        value: groupName
                    }
                )
            });
            return;
        }

        actionStatusRef.current = 'local-favorite';
        setActionStatus('local-favorite');
        try {
            if (kind === 'world' && isRecord(entity)) {
                persistWorldDetails(entity, normalizedEntityId);
            } else if (kind === 'avatar' && isRecord(entity)) {
                persistAvatarDetails(entity, normalizedEntityId);
            }
            await favoritePersistenceRepository.createLocalFavoriteGroup({
                kind,
                groupName
            });
            await favoritePersistenceRepository.addLocalFavorite({
                kind,
                entityId: normalizedEntityId,
                groupName
            });
            toast.add({
                type: 'success',
                title: t('view.favorite.label.local_favorite_added')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'view.favorites.toast.failed_to_create_local_favorite_group'
                          )
            });
        } finally {
            actionStatusRef.current = 'idle';
            setActionStatus('idle');
        }
    }

    if (!normalizedEntityId) {
        return null;
    }

    const favorited = Boolean(remoteFavorite) || localFavoriteActive;
    const triggerLabel = favorited
        ? t('view.favorite.label.favorited')
        : label || t('view.favorite.label.favorite');
    const localFavoritesLabel =
        kind === 'avatar'
            ? t('dialog.favorite.local_avatar_favorites')
            : t('dialog.favorite.local_favorites');

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={
                    <Button
                        type="button"
                        size={iconOnly ? 'icon-lg' : 'sm'}
                        variant={
                            iconOnly
                                ? 'outline'
                                : remoteFavorite
                                  ? 'default'
                                  : 'outline'
                        }
                        disabled={actionStatus !== 'idle'}
                        aria-label={triggerLabel}
                        title={triggerLabel}
                    >
                        {actionStatus !== 'idle' ? (
                            <Spinner data-icon="inline-start" />
                        ) : (
                            <StarIcon
                                data-icon="inline-start"
                                className={favorited ? 'fill-current' : ''}
                            />
                        )}
                        {iconOnly ? null : triggerLabel}
                    </Button>
                }
            />
            <DropdownMenuContent align="start" className="w-64">
                <DropdownMenuGroup>
                    <DropdownMenuLabel>
                        {t('view.favorite.label.vrchat_favorites')}
                    </DropdownMenuLabel>
                    {remoteFavorite && !remoteFavoriteGroup ? (
                        <DropdownMenuCheckboxItem
                            checked
                            onClick={(event) => event.preventDefault()}
                            onCheckedChange={() =>
                                void setRemoteFavoriteGroup(null)
                            }
                        >
                            {remoteFavoriteGroupKey(remoteFavorite) ||
                                remoteFavoriteTag(remoteFavorite) ||
                                'Current group'}
                        </DropdownMenuCheckboxItem>
                    ) : null}
                    {groups.length ? (
                        groups.map((group) => {
                            const isCurrent = group === remoteFavoriteGroup;
                            const isFull =
                                Number(group.capacity) > 0 &&
                                (Number(group.count) || 0) >=
                                    Number(group.capacity);

                            return (
                                <DropdownMenuCheckboxItem
                                    key={String(group.key ?? '')}
                                    checked={isCurrent}
                                    disabled={isFull && !isCurrent}
                                    onClick={(event) => event.preventDefault()}
                                    onCheckedChange={() =>
                                        void setRemoteFavoriteGroup(
                                            isCurrent ? null : group
                                        )
                                    }
                                >
                                    {formatGroupLabel(group)}
                                </DropdownMenuCheckboxItem>
                            );
                        })
                    ) : remoteFavorite ? null : (
                        <DropdownMenuItem disabled>
                            {t('view.favorite.empty.no_favorite_groups_loaded')}
                        </DropdownMenuItem>
                    )}
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                    <DropdownMenuLabel>{localFavoritesLabel}</DropdownMenuLabel>
                    {localGroups.length ? (
                        localGroups.map((groupName) => {
                            const isLocalFavorite = hasLocalFavorite(
                                localFavorites,
                                groupName,
                                normalizedEntityId
                            );
                            return (
                                <DropdownMenuCheckboxItem
                                    key={groupName}
                                    checked={isLocalFavorite}
                                    onClick={(event) => event.preventDefault()}
                                    onCheckedChange={() => {
                                        if (isLocalFavorite) {
                                            removeLocalFavoriteFromGroup(
                                                groupName
                                            );
                                        } else {
                                            addLocalFavoriteToGroup(groupName);
                                        }
                                    }}
                                >
                                    {localGroupLabel(localFavorites, groupName)}
                                </DropdownMenuCheckboxItem>
                            );
                        })
                    ) : (
                        <DropdownMenuItem disabled>
                            {t(
                                'view.favorite.empty.no_local_favorite_groups_loaded'
                            )}
                        </DropdownMenuItem>
                    )}
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                    <DropdownMenuItem
                        onClick={() => void createLocalFavoriteGroupAndAdd()}
                    >
                        <PlusIcon data-icon="inline-start" />
                        {t('view.favorite.worlds.new_group')}
                    </DropdownMenuItem>
                </DropdownMenuGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
