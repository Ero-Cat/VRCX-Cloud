import { UserIcon, UserPlusIcon, XIcon } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { commands, type WatchedUserOutput } from '@/platform/native/bindings';
import configRepository from '@/repositories/configRepository';
import userProfileRepository from '@/repositories/userProfileRepository';
import { openUserDialog } from '@/services/dialogService';
import { toast } from '@/services/toastService';
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
import { Input } from '@/ui/shadcn/input';
import { Spinner } from '@/ui/shadcn/spinner';
import { Switch } from '@/ui/shadcn/switch';

const PROFILE_WATCH_CONFIG_KEY = 'profileWatchEnabled';

export function TrackedUsersSidebar() {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const [rows, setRows] = useState<WatchedUserOutput[]>([]);
    const [pollEnabled, setPollEnabled] = useState(false);
    const [addDialogOpen, setAddDialogOpen] = useState(false);

    const reload = useCallback(async () => {
        if (!currentUserId) {
            setRows([]);
            return;
        }
        try {
            const [list, toggle] = await Promise.all([
                commands.appWatchedUsersList(currentUserId),
                configRepository.getBool(PROFILE_WATCH_CONFIG_KEY, false)
            ]);
            setRows(list);
            setPollEnabled(Boolean(toggle));
        } catch {
            setRows([]);
        }
    }, [currentUserId]);

    useEffect(() => {
        void reload();
    }, [reload]);

    async function changePollEnabled(next: boolean) {
        setPollEnabled(next);
        try {
            await configRepository.setBool(PROFILE_WATCH_CONFIG_KEY, next);
        } catch {
            setPollEnabled(!next);
        }
    }

    async function removeEntry(userId: string) {
        if (!currentUserId) {
            return;
        }
        try {
            await commands.appWatchedUserRemove({
                ownerUserId: currentUserId,
                targetUserId: userId
            });
            await reload();
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('view.tools.watched_users.failed')
            });
        }
    }

    return (
        <div className="relative h-full">
            <div className="h-full w-full overflow-auto overflow-x-hidden">
                <div className="flex items-center justify-between px-3 py-2">
                    <span className="text-muted-foreground text-xs">
                        {t('side_panel.tracked_nonfriends.poll_toggle')}
                    </span>
                    <Switch
                        checked={pollEnabled}
                        onCheckedChange={changePollEnabled}
                    />
                </div>
                {rows.length === 0 ? (
                    <div className="text-muted-foreground flex flex-col items-center justify-center px-4 py-8 text-center text-xs">
                        <span>{t('side_panel.tracked_nonfriends.empty')}</span>
                    </div>
                ) : (
                    <div className="flex flex-col gap-0.5 px-1.5 pb-16">
                        {rows.map((entry) => (
                            <button
                                type="button"
                                key={entry.userId}
                                className="group hover:bg-muted/50 box-border flex w-full cursor-pointer items-center rounded-lg p-1.5 text-left text-[13px]"
                                onClick={() =>
                                    openUserDialog({
                                        userId: entry.userId,
                                        title: entry.displayName || entry.userId
                                    })
                                }
                            >
                                <span className="bg-muted flex size-9 flex-none items-center justify-center overflow-hidden rounded-full border">
                                    <UserIcon className="text-muted-foreground size-5" />
                                </span>
                                <div className="mx-2.5 flex h-9 flex-1 flex-col justify-between overflow-hidden">
                                    <span className="truncate leading-[18px] font-medium">
                                        {entry.displayName || entry.userId}
                                    </span>
                                    <span className="text-muted-foreground truncate text-xs">
                                        {entry.lastStatusDescription || ''}
                                    </span>
                                </div>
                                <Button
                                    type="button"
                                    size="icon-sm"
                                    variant="ghost"
                                    className="ml-1 flex-none opacity-0 group-hover:opacity-100"
                                    aria-label={t(
                                        'side_panel.tracked_nonfriends.remove_tooltip'
                                    )}
                                    onClick={(event) => {
                                        event.stopPropagation();
                                        void removeEntry(entry.userId);
                                    }}
                                >
                                    <XIcon className="size-3.5" />
                                </Button>
                            </button>
                        ))}
                    </div>
                )}
            </div>

            <div className="absolute right-4 bottom-5 z-10">
                <Button
                    type="button"
                    size="sm"
                    className="rounded-full shadow-md"
                    onClick={() => setAddDialogOpen(true)}
                >
                    {t('side_panel.tracked_nonfriends.add_button')}
                    <UserPlusIcon
                        data-icon="inline-start"
                        className="size-3.5"
                    />
                </Button>
            </div>

            <TrackedAddDialog
                open={addDialogOpen}
                onOpenChange={setAddDialogOpen}
                onAdded={reload}
            />
        </div>
    );
}

function TrackedAddDialog({
    open,
    onOpenChange,
    onAdded
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onAdded: () => Promise<void>;
}) {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const [step, setStep] = useState<'input' | 'confirm'>('input');
    const [input, setInput] = useState('');
    const [verifiedName, setVerifiedName] = useState('');
    const [verifying, setVerifying] = useState(false);
    const [error, setError] = useState('');
    const requestIdRef = useRef(0);

    useEffect(() => {
        if (!open) {
            setStep('input');
            setInput('');
            setVerifiedName('');
            setError('');
        }
    }, [open]);

    async function verifyUser() {
        const userId = input.trim();
        if (!currentUserId || !userId.startsWith('usr_')) {
            setError(t('side_panel.tracked_nonfriends.add_input_invalid'));
            return;
        }
        const requestId = ++requestIdRef.current;
        setVerifying(true);
        setError('');
        try {
            const profile = await userProfileRepository.getUserProfile({
                userId
            });
            if (requestId !== requestIdRef.current) {
                return;
            }
            setVerifiedName(profile?.displayName || '');
            setStep('confirm');
        } catch (fetchError) {
            if (requestId !== requestIdRef.current) {
                return;
            }
            setError(
                fetchError instanceof Error
                    ? fetchError.message
                    : t('view.tools.watched_users.failed')
            );
        } finally {
            if (requestId === requestIdRef.current) {
                setVerifying(false);
            }
        }
    }

    async function confirmAdd() {
        const userId = input.trim();
        if (!currentUserId || !userId) {
            return;
        }
        try {
            await commands.appWatchedUserAdd({
                ownerUserId: currentUserId,
                targetUserId: userId,
                displayName: verifiedName
            });
            await onAdded();
            onOpenChange(false);
        } catch (addError) {
            setError(
                addError instanceof Error
                    ? addError.message
                    : t('view.tools.watched_users.failed')
            );
            setStep('input');
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="w-90 max-w-[95vw]">
                <DialogHeader>
                    <DialogTitle>
                        {t('side_panel.tracked_nonfriends.add_dialog_title')}
                    </DialogTitle>
                    <DialogDescription>
                        {t('side_panel.tracked_nonfriends.add_dialog_hint')}
                    </DialogDescription>
                </DialogHeader>
                {step === 'input' ? (
                    <>
                        <div className="flex flex-col gap-3 py-2">
                            <Input
                                value={input}
                                placeholder={t(
                                    'side_panel.tracked_nonfriends.add_input_placeholder'
                                )}
                                disabled={verifying}
                                onChange={(event) =>
                                    setInput(event.target.value)
                                }
                                onKeyDown={(event) => {
                                    if (event.key === 'Enter') {
                                        void verifyUser();
                                    }
                                }}
                            />
                            {error ? (
                                <p className="text-destructive text-sm">
                                    {error}
                                </p>
                            ) : null}
                        </div>
                        <DialogFooter>
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => onOpenChange(false)}
                            >
                                {t('common.actions.cancel')}
                            </Button>
                            <Button
                                type="button"
                                disabled={!input.trim() || verifying}
                                onClick={verifyUser}
                            >
                                {verifying ? <Spinner /> : null}
                                {t('common.actions.confirm')}
                            </Button>
                        </DialogFooter>
                    </>
                ) : (
                    <>
                        <div className="flex items-center gap-3 py-2">
                            <span className="bg-muted flex size-12 flex-none items-center justify-center overflow-hidden rounded-full border">
                                <UserIcon className="text-muted-foreground size-5" />
                            </span>
                            <div>
                                <p className="font-medium">
                                    {t(
                                        'side_panel.tracked_nonfriends.add_confirm_question',
                                        {
                                            name: verifiedName || input
                                        }
                                    )}
                                </p>
                                <p className="text-muted-foreground text-xs">
                                    {input}
                                </p>
                            </div>
                        </div>
                        {error ? (
                            <p className="text-destructive text-sm">{error}</p>
                        ) : null}
                        <DialogFooter>
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => setStep('input')}
                            >
                                {t('side_panel.tracked_nonfriends.add_back')}
                            </Button>
                            <Button type="button" onClick={confirmAdd}>
                                {t('common.actions.confirm')}
                            </Button>
                        </DialogFooter>
                    </>
                )}
            </DialogContent>
        </Dialog>
    );
}
