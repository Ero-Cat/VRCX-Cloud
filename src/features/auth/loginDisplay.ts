import type { SavedCredentialSnapshot } from '@/platform/native/bindings';

type LoginUserRecord = {
    displayName?: string | null;
    id?: string | null;
    username?: string | null;
};

export function getLoginUserDisplayName(
    user: LoginUserRecord | null | undefined
): string {
    return user?.displayName || user?.username || user?.id || 'account';
}

export function shouldShowLegacyMigrationAction(
    isLoading: boolean,
    savedAccounts: ReadonlyArray<Pick<SavedCredentialSnapshot, 'user'>>
): boolean {
    return !isLoading && savedAccounts.length === 0;
}
