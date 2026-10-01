import { ArchiveRestoreIcon, NetworkIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';
import { Spinner } from '@/ui/shadcn/spinner';

type LoginPageUtilitiesProps = {
    disabled: boolean;
    isValidatingRestore: boolean;
    onOpenProxyDialog: () => void;
    onRestoreProfileBackup: () => void;
};

export function LoginPageUtilities({
    disabled,
    isValidatingRestore,
    onOpenProxyDialog,
    onRestoreProfileBackup
}: LoginPageUtilitiesProps) {
    const { t } = useTranslation();

    return (
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <Button type="button" variant="outline" onClick={onOpenProxyDialog}>
                <NetworkIcon data-icon="inline-start" />
                {t('view.login.proxy_settings')}
            </Button>
            <Button
                type="button"
                variant="outline"
                disabled={disabled || isValidatingRestore}
                onClick={onRestoreProfileBackup}
            >
                {isValidatingRestore ? (
                    <Spinner data-icon="inline-start" />
                ) : (
                    <ArchiveRestoreIcon data-icon="inline-start" />
                )}
                {t('profile_backup.restore_from_backup')}
            </Button>
        </div>
    );
}
