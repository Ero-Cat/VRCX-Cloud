import { PlusIcon, UserIcon, UsersIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Location } from '@/components/Location';
import {
    FriendMultiSelectList,
    type FriendMultiSelectOption
} from '@/components/search/FriendMultiSelectList';
import worldProfileRepository from '@/repositories/worldProfileRepository';
import { sendInvitesToLocation } from '@/services/inviteDeliveryService';
import { toast } from '@/services/toastService';
import { parseLocation } from '@/shared/utils/location';
import { normalizeString as normalizeId } from '@/shared/utils/string';
import { useFavoriteStore } from '@/state/favoriteStore';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useModalStore } from '@/state/modalStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Button } from '@/ui/shadcn/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
} from '@/ui/shadcn/dialog';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger
} from '@/ui/shadcn/dropdown-menu';
import {
    Empty,
    EmptyDescription,
    EmptyHeader,
    EmptyTitle
} from '@/ui/shadcn/empty';
import { Spinner } from '@/ui/shadcn/spinner';

import {
    buildFavoriteGroupItems,
    buildFavoriteGroupLabelsByUserId,
    buildFriendsInCurrentInstanceIds,
    displayNameForUser
} from './inviteDialogModel';

export function InstanceInviteDialog({
    open,
    location = '',
    launchToken = '',
    worldName = '',
    endpoint = '',
    onOpenChange
}: {
    open: boolean;
    location?: string;
    launchToken?: string;
    worldName?: string;
    endpoint?: string;
    onOpenChange(open: boolean): void;
}) {
    const { t } = useTranslation();

    const currentUser = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentLocationPlayerIds = useRuntimeStore(
        (state) => state.gameState.currentLocationPlayerIds
    );
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const onlineIds = useFriendRosterStore((state) => state.onlineIds);
    const activeIds = useFriendRosterStore((state) => state.activeIds);
    const favoriteFriendGroups = useFavoriteStore(
        (state) => state.favoriteFriendGroups
    );
    const groupedFavoriteFriendIdsByGroupKey = useFavoriteStore(
        (state) => state.groupedFavoriteFriendIdsByGroupKey
    );
    const localFriendFavoriteGroups = useFavoriteStore(
        (state) => state.localFriendFavoriteGroups
    );
    const localFriendFavorites = useFavoriteStore(
        (state) => state.localFriendFavorites
    );
    const confirm = useModalStore((state) => state.confirm);
    const [selectedUserIds, setSelectedUserIds] = useState<string[]>([]);
    const [sending, setSending] = useState(false);
    const [resolvedWorldName, setResolvedWorldName] = useState('');

    useEffect(() => {
        if (open) {
            setSelectedUserIds([]);
            setSending(false);
        }
    }, [open, location]);

    useEffect(() => {
        let active = true;
        const nextWorldName = normalizeId(worldName);
        setResolvedWorldName(nextWorldName);
        if (!open || nextWorldName) {
            return () => {
                active = false;
            };
        }

        const parsedLocation = parseLocation(location);
        if (!parsedLocation.worldId) {
            return () => {
                active = false;
            };
        }

        worldProfileRepository
            .getWorldProfile({
                worldId: parsedLocation.worldId
            })
            .then((world) => {
                if (active) {
                    setResolvedWorldName(normalizeId(world?.name));
                }
            })
            .catch(() => {});
        return () => {
            active = false;
        };
    }, [endpoint, location, open, worldName]);

    const selectableUserIds = useMemo(() => {
        const ids = [];
        if (currentUserId) {
            ids.push(currentUserId);
        }
        for (const userId of [...onlineIds, ...activeIds]) {
            const normalizedUserId = normalizeId(userId);
            if (normalizedUserId && !ids.includes(normalizedUserId)) {
                ids.push(normalizedUserId);
            }
        }
        return ids;
    }, [activeIds, currentUserId, onlineIds]);

    const favoriteGroupLabelsByUserId = useMemo(
        () =>
            buildFavoriteGroupLabelsByUserId({
                favoriteFriendGroups,
                groupedFavoriteFriendIdsByGroupKey,
                localFriendFavoriteGroups,
                localFriendFavorites
            }),
        [
            favoriteFriendGroups,
            groupedFavoriteFriendIdsByGroupKey,
            localFriendFavoriteGroups,
            localFriendFavorites
        ]
    );

    const inviteOptions = useMemo<FriendMultiSelectOption[]>(
        () =>
            selectableUserIds.map((userId) => {
                const label = displayNameForUser(
                    userId,
                    friendsById,
                    currentUser
                );
                const [firstGroup, ...restGroups] =
                    favoriteGroupLabelsByUserId[userId] || [];
                return {
                    value: userId,
                    label,
                    search: `${label} ${userId}`,
                    user:
                        friendsById[userId] ??
                        (userId === normalizeId(currentUser?.id)
                            ? currentUser
                            : null),
                    badge:
                        firstGroup && restGroups.length
                            ? `${firstGroup} +${restGroups.length}`
                            : firstGroup
                };
            }),
        [
            currentUser,
            favoriteGroupLabelsByUserId,
            friendsById,
            selectableUserIds
        ]
    );

    const friendsInCurrentInstanceIds = useMemo(
        () =>
            buildFriendsInCurrentInstanceIds({
                currentLocationPlayerIds,
                friendsById
            }),
        [currentLocationPlayerIds, friendsById]
    );

    const favoriteGroupItems = useMemo(
        () =>
            buildFavoriteGroupItems({
                favoriteFriendGroups,
                groupedFavoriteFriendIdsByGroupKey,
                localFriendFavoriteGroups,
                localFriendFavorites,
                friendsById
            }),
        [
            favoriteFriendGroups,
            friendsById,
            groupedFavoriteFriendIdsByGroupKey,
            localFriendFavoriteGroups,
            localFriendFavorites
        ]
    );

    function addUserIds(userIds: string[]) {
        const ids = userIds.map(normalizeId).filter(Boolean);
        if (!ids.length) {
            return;
        }
        setSelectedUserIds((current) => [...new Set([...current, ...ids])]);
    }

    async function sendInvites() {
        const parsedLocation = parseLocation(location);
        const normalizedUserIds = selectedUserIds
            .map(normalizeId)
            .filter(Boolean);
        if (!parsedLocation.worldId || !parsedLocation.instanceId) {
            toast.add({
                type: 'error',
                title: t(
                    'dialog.invite.error.cannot_invite_location_is_not_a_concrete_instance'
                )
            });
            return;
        }
        if (!normalizedUserIds.length) {
            toast.add({
                type: 'error',
                title: t(
                    'dialog.invite.action.select_at_least_one_user_to_invite'
                )
            });
            return;
        }

        const result = await confirm({
            title: t('dialog.instance_invite.modal.send_invite'),
            description: t(
                normalizedUserIds.length === 1
                    ? 'dialog.instance_invite.dynamic.send_invite_to_value_user'
                    : 'dialog.instance_invite.dynamic.send_invites_to_value_users',
                {
                    value: normalizedUserIds.length
                }
            ),
            confirmText: t('dialog.instance_invite.modal.invite'),
            cancelText: t('common.actions.cancel')
        });
        if (!result.ok) {
            return;
        }

        setSending(true);
        try {
            const batch = await sendInvitesToLocation({
                receiverUserIds: normalizedUserIds,
                location: parsedLocation.tag || location,
                shortName: launchToken || parsedLocation.shortName,
                worldName:
                    resolvedWorldName || worldName || parsedLocation.worldId
            });
            const failedItems = batch.items.filter(
                (item) => item.state === 'failed'
            );
            const failedUserIds = new Set(
                failedItems.map((item) => item.receiverUserId)
            );
            const failures = failedItems.map((item) => item.message);
            const successCount = batch.succeeded;

            if (successCount) {
                toast.add({
                    type: 'success',
                    title:
                        successCount === 1
                            ? t('message.invite.sent')
                            : t(
                                  'dialog.instance_invite.toast.sent_value_invites',
                                  {
                                      value: successCount
                                  }
                              )
                });
            }
            if (failures.length) {
                setSelectedUserIds((current) =>
                    current.filter((userId) => failedUserIds.has(userId))
                );
                toast.add({
                    type: 'error',
                    title:
                        failures.length === 1
                            ? failures[0]
                            : t(
                                  'dialog.instance_invite.toast.failed_to_send_value_invites',
                                  { value: failures.length }
                              )
                });
            } else {
                onOpenChange?.(false);
            }
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'dialog.instance_invite.toast.failed_to_send_invite'
                          )
            });
        } finally {
            setSending(false);
        }
    }

    return (
        <Dialog open={Boolean(open)} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{t('dialog.invite.invite')}</DialogTitle>
                    <DialogDescription>
                        {t(
                            'dialog.invite.description.choose_online_friends_to_invite_to_this_instance'
                        )}
                    </DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-4">
                    <div className="bg-muted/30 rounded-md border p-3 text-sm">
                        <Location
                            location={location}
                            link={false}
                            asButton={false}
                            className="cursor-default"
                        />
                    </div>
                    <div className="flex flex-wrap gap-2">
                        <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            disabled={!currentUserId || sending}
                            onClick={() => {
                                if (currentUserId) {
                                    addUserIds([currentUserId]);
                                }
                            }}
                        >
                            <UserIcon data-icon="inline-start" />
                            {t('dialog.invite.add_self')}
                        </Button>
                        <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            disabled={
                                !friendsInCurrentInstanceIds.length || sending
                            }
                            onClick={() =>
                                addUserIds(friendsInCurrentInstanceIds)
                            }
                        >
                            <UsersIcon data-icon="inline-start" />
                            {t('dialog.invite.add_friends_in_instance')}
                        </Button>
                        <DropdownMenu>
                            <DropdownMenuTrigger
                                render={
                                    <Button
                                        type="button"
                                        size="sm"
                                        variant="outline"
                                        disabled={
                                            sending ||
                                            (!favoriteGroupItems.remote
                                                .length &&
                                                !favoriteGroupItems.local
                                                    .length)
                                        }
                                    >
                                        <PlusIcon data-icon="inline-start" />
                                        {t(
                                            'dialog.invite.add_favorite_friends'
                                        )}
                                    </Button>
                                }
                            />
                            <DropdownMenuContent align="start" className="w-56">
                                <DropdownMenuGroup>
                                    {favoriteGroupItems.remote.map((group) => (
                                        <DropdownMenuItem
                                            key={group.key}
                                            onClick={() =>
                                                addUserIds(group.userIds)
                                            }
                                        >
                                            {group.label}
                                        </DropdownMenuItem>
                                    ))}
                                </DropdownMenuGroup>
                                {favoriteGroupItems.remote.length &&
                                favoriteGroupItems.local.length ? (
                                    <DropdownMenuSeparator />
                                ) : null}
                                <DropdownMenuGroup>
                                    {favoriteGroupItems.local.map((group) => (
                                        <DropdownMenuItem
                                            key={group.key}
                                            onClick={() =>
                                                addUserIds(group.userIds)
                                            }
                                        >
                                            {group.label}
                                        </DropdownMenuItem>
                                    ))}
                                </DropdownMenuGroup>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                    <FriendMultiSelectList
                        autoFocus
                        disabled={sending}
                        options={inviteOptions}
                        values={selectedUserIds}
                        onChange={setSelectedUserIds}
                        placeholder={t(
                            'dialog.invite.action.search_online_friends'
                        )}
                        listClassName="h-64"
                        emptyContent={
                            <Empty className="min-h-32 border-0">
                                <EmptyHeader>
                                    <EmptyTitle>
                                        {t(
                                            'dialog.invite.empty.no_online_friends'
                                        )}
                                    </EmptyTitle>
                                    <EmptyDescription>
                                        {t(
                                            'dialog.invite.empty.no_selectable_online_friends_match_the_current_search'
                                        )}
                                    </EmptyDescription>
                                </EmptyHeader>
                            </Empty>
                        }
                    />
                </div>
                <DialogFooter>
                    <Button
                        type="button"
                        variant="outline"
                        disabled={sending}
                        onClick={() => onOpenChange?.(false)}
                    >
                        {t('common.actions.cancel')}
                    </Button>
                    <Button
                        type="button"
                        disabled={sending || !selectedUserIds.length}
                        onClick={() => {
                            sendInvites();
                        }}
                    >
                        {sending ? <Spinner data-icon="inline-start" /> : null}
                        {t('dialog.invite.invite')}
                        {selectedUserIds.length
                            ? ` (${selectedUserIds.length})`
                            : null}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
