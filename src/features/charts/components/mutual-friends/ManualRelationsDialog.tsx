import { PlusIcon, Trash2Icon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { displayNameForUser } from '@/components/dialogs/inviteDialogModel';
import { FriendMultiSelectList } from '@/components/search/FriendMultiSelectList';
import type { FriendMultiSelectOption } from '@/components/search/FriendMultiSelectList';
import mutualGraphPersistenceRepository, {
    type MutualGraphManualLink
} from '@/repositories/mutualGraphPersistenceRepository';
import { openUserDialog } from '@/services/dialogService';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Button } from '@/ui/shadcn/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle
} from '@/ui/shadcn/dialog';
import { Label } from '@/ui/shadcn/label';

function formatAddedAt(addedAt: string): string {
    const ms = Date.parse(addedAt);
    if (!Number.isFinite(ms)) {
        return '';
    }
    const date = new Date(ms);
    const pad = (value: number) => String(value).padStart(2, '0');
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function ManualRelationsDialog({
    open,
    onOpenChange,
    manualLinks,
    onChanged
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    manualLinks: MutualGraphManualLink[];
    onChanged: () => void;
}) {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentUser = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const [userA, setUserA] = useState('');
    const [userB, setUserB] = useState('');
    const [error, setError] = useState('');

    useEffect(() => {
        if (!open) {
            setUserA('');
            setUserB('');
            setError('');
        }
    }, [open]);

    const options = useMemo<FriendMultiSelectOption[]>(() => {
        const items: FriendMultiSelectOption[] = [];
        for (const friendId of Object.keys(friendsById)) {
            const label = displayNameForUser(
                friendId,
                friendsById,
                currentUser
            );
            items.push({
                value: friendId,
                label,
                search: `${label} ${friendId}`,
                user: friendsById[friendId] ?? null
            });
        }
        items.sort((left, right) => left.label.localeCompare(right.label));
        return items;
    }, [friendsById, currentUser]);

    const optionsA = useMemo(
        () => options.filter((option) => option.value !== userB),
        [options, userB]
    );
    const optionsB = useMemo(
        () => options.filter((option) => option.value !== userA),
        [options, userA]
    );

    const linkPairs = useMemo(
        () => new Set(manualLinks.map((link) => `${link.left}__${link.right}`)),
        [manualLinks]
    );

    function resolveName(userId: string): string {
        return (
            friendsById[userId]?.displayName ||
            displayNameForUser(userId, friendsById, currentUser) ||
            userId
        );
    }

    async function addRelation() {
        if (!currentUserId || !userA || !userB) {
            return;
        }
        const [left, right] = [userA, userB].sort();
        if (linkPairs.has(`${left}__${right}`)) {
            setError(
                t('view.charts.mutual_friend.manual_relations.already_exists')
            );
            return;
        }
        setError('');
        try {
            await mutualGraphPersistenceRepository.addManualLink(
                currentUserId,
                userA,
                userB
            );
            setUserA('');
            setUserB('');
            onChanged();
        } catch (addError) {
            setError(
                addError instanceof Error ? addError.message : String(addError)
            );
        }
    }

    async function deleteRelation(left: string, right: string) {
        if (!currentUserId) {
            return;
        }
        try {
            await mutualGraphPersistenceRepository.removeManualLink(
                currentUserId,
                left,
                right
            );
            onChanged();
        } catch {
            // The list refresh on next reload surfaces any failure.
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="flex max-h-[85vh] max-w-2xl flex-col overflow-hidden">
                <DialogHeader>
                    <DialogTitle>
                        {t(
                            'view.charts.mutual_friend.manual_relations.dialog_title'
                        )}
                    </DialogTitle>
                    <DialogDescription>
                        {t(
                            'view.charts.mutual_friend.manual_relations.list_empty'
                        )}
                    </DialogDescription>
                </DialogHeader>

                <div className="bg-muted/30 flex flex-col gap-3 rounded-md border p-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div className="flex flex-col gap-1.5">
                            <Label>
                                {t(
                                    'view.charts.mutual_friend.manual_relations.user_a'
                                )}
                            </Label>
                            <FriendMultiSelectList
                                options={optionsA}
                                values={userA ? [userA] : []}
                                onChange={(values) =>
                                    setUserA(values[values.length - 1] ?? '')
                                }
                                placeholder={t(
                                    'view.charts.mutual_friend.manual_relations.user_placeholder'
                                )}
                                emptyContent={t(
                                    'view.charts.mutual_friend.manual_relations.user_search'
                                )}
                            />
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label>
                                {t(
                                    'view.charts.mutual_friend.manual_relations.user_b'
                                )}
                            </Label>
                            <FriendMultiSelectList
                                options={optionsB}
                                values={userB ? [userB] : []}
                                onChange={(values) =>
                                    setUserB(values[values.length - 1] ?? '')
                                }
                                placeholder={t(
                                    'view.charts.mutual_friend.manual_relations.user_placeholder'
                                )}
                                emptyContent={t(
                                    'view.charts.mutual_friend.manual_relations.user_search'
                                )}
                            />
                        </div>
                    </div>
                    <div className="flex items-center justify-between">
                        <span className="text-destructive text-xs">
                            {error}
                        </span>
                        <Button
                            type="button"
                            disabled={!userA || !userB || userA === userB}
                            onClick={addRelation}
                        >
                            <PlusIcon data-icon="inline-start" />
                            {t(
                                'view.charts.mutual_friend.manual_relations.add_button'
                            )}
                        </Button>
                    </div>
                </div>

                <div className="flex-1 overflow-y-auto pr-1">
                    {manualLinks.length === 0 ? (
                        <div className="text-muted-foreground py-8 text-center italic">
                            {t(
                                'view.charts.mutual_friend.manual_relations.list_empty'
                            )}
                        </div>
                    ) : (
                        <div className="flex flex-col gap-2 pb-2">
                            {manualLinks.map((link) => (
                                <div
                                    key={`${link.left}__${link.right}`}
                                    className="bg-card flex items-center justify-between gap-3 rounded-md border p-3 shadow-sm"
                                >
                                    <div className="flex min-w-0 items-center gap-3">
                                        <button
                                            type="button"
                                            className="hover:text-primary truncate font-medium underline underline-offset-2 transition-colors"
                                            onClick={() =>
                                                openUserDialog({
                                                    userId: link.left,
                                                    title: resolveName(
                                                        link.left
                                                    )
                                                })
                                            }
                                        >
                                            {resolveName(link.left)}
                                        </button>
                                        <span className="text-muted-foreground shrink-0 text-xs">
                                            ↔
                                        </span>
                                        <button
                                            type="button"
                                            className="hover:text-primary truncate font-medium underline underline-offset-2 transition-colors"
                                            onClick={() =>
                                                openUserDialog({
                                                    userId: link.right,
                                                    title: resolveName(
                                                        link.right
                                                    )
                                                })
                                            }
                                        >
                                            {resolveName(link.right)}
                                        </button>
                                    </div>
                                    <div className="flex shrink-0 items-center gap-4">
                                        <span className="text-muted-foreground text-[11px]">
                                            {formatAddedAt(link.createdAt)}
                                        </span>
                                        <Button
                                            type="button"
                                            size="icon"
                                            variant="ghost"
                                            className="text-destructive hover:bg-destructive/10 h-8 w-8"
                                            aria-label={t(
                                                'common.actions.remove'
                                            )}
                                            onClick={() =>
                                                deleteRelation(
                                                    link.left,
                                                    link.right
                                                )
                                            }
                                        >
                                            <Trash2Icon className="size-4" />
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </DialogContent>
        </Dialog>
    );
}
