import { ShieldCheckIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useTranslation } from 'react-i18next';

import {
    useAdminAuthStore,
    type AdminUnlockFailure
} from '@/state/adminAuthStore';
import { Button } from '@/ui/shadcn/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
} from '@/ui/shadcn/dialog';
import { Field, FieldError, FieldGroup, FieldLabel } from '@/ui/shadcn/field';
import { Input } from '@/ui/shadcn/input';
import { Spinner } from '@/ui/shadcn/spinner';

/**
 * Second-factor gate for the self-hosted server: a single dialog asking
 * for the admin password when the server has one configured and this
 * browser has not unlocked yet. The unlock cookie is permanent, so the
 * dialog appears at most once per browser.
 */
export function AdminAuthGate() {
    const phase = useAdminAuthStore((state) => state.phase);
    const refresh = useAdminAuthStore((state) => state.refresh);

    useEffect(() => {
        void refresh();
    }, [refresh]);

    return <AdminAuthDialog open={phase === 'locked'} />;
}

function AdminAuthDialog({ open }: { open: boolean }) {
    const { t } = useTranslation();
    const unlock = useAdminAuthStore((state) => state.unlock);
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const [submitting, setSubmitting] = useState(false);

    function failureMessage(failure: AdminUnlockFailure): string {
        return failure === 'wrongPassword'
            ? t('admin_auth.error.wrong_password')
            : t('admin_auth.error.failed');
    }

    async function handleSubmit(event: FormEvent<HTMLFormElement>) {
        event.preventDefault();
        if (submitting || !password) {
            return;
        }
        setSubmitting(true);
        setError('');
        const failure = await unlock(password);
        if (failure === null) {
            // The unlock cookie is set: reboot so every boot service
            // re-runs against the now-unauthenticated-free API.
            window.location.reload();
            return;
        }
        setSubmitting(false);
        setPassword('');
        setError(failureMessage(failure));
    }

    return (
        <Dialog
            open={open}
            onOpenChange={() => {
                // Not dismissable: the whole web API stays gated until
                // this browser unlocks.
            }}
        >
            <DialogContent showCloseButton={false}>
                <form className="contents" onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <ShieldCheckIcon className="size-4" />
                            {t('admin_auth.title')}
                        </DialogTitle>
                        <DialogDescription>
                            {t('admin_auth.description')}
                        </DialogDescription>
                    </DialogHeader>
                    <FieldGroup className="gap-3">
                        <Field data-invalid={Boolean(error) || undefined}>
                            <FieldLabel htmlFor="admin-auth-password">
                                {t('admin_auth.field.password')}
                            </FieldLabel>
                            <Input
                                id="admin-auth-password"
                                type="password"
                                autoComplete="current-password"
                                autoFocus
                                aria-invalid={Boolean(error) || undefined}
                                disabled={submitting}
                                value={password}
                                onChange={(event) => {
                                    setPassword(event.target.value);
                                    setError('');
                                }}
                            />
                            <FieldError>{error}</FieldError>
                        </Field>
                    </FieldGroup>
                    <DialogFooter>
                        <Button
                            type="submit"
                            className="w-full sm:w-auto"
                            disabled={submitting || !password}
                        >
                            {submitting ? (
                                <Spinner data-icon="inline-start" />
                            ) : null}
                            {t('admin_auth.action.unlock')}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
