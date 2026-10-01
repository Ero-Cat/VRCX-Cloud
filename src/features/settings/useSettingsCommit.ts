import { useTranslation } from 'react-i18next';

import { toast } from '@/services/toastService';

type SettingsCommitAction = () => void;
type SettingsRollback = () => void;
type SettingsOptimisticUpdate = () => void | SettingsRollback;

export type SettingsCommit = (
    action: SettingsCommitAction,
    optimistic?: SettingsOptimisticUpdate
) => Promise<boolean>;

export function useSettingsCommit(): SettingsCommit {
    const { t } = useTranslation();

    return async function commit(action, optimistic) {
        const rollback = optimistic?.();
        try {
            await action();
            return true;
        } catch (error) {
            rollback?.();
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('view.settings.toast.failed_to_save_setting')
            });
            return false;
        }
    };
}
