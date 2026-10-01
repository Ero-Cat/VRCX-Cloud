import { useTranslation } from 'react-i18next';

import { IndeterminateProgress } from '@/components/IndeterminateProgress';
import { useCriticalTask } from '@/lib/useCriticalTask';
import { useRuntimeStore } from '@/state/runtimeStore';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle
} from '@/ui/shadcn/dialog';

export function DatabaseMaintenanceDialog() {
    const { t } = useTranslation();
    const active = useRuntimeStore((state) => state.databaseMaintenanceActive);
    useCriticalTask('databaseMaintenance', active);

    return (
        <Dialog open={active} onOpenChange={() => undefined}>
            <DialogContent showCloseButton={false}>
                <DialogHeader>
                    <DialogTitle>
                        {t('message.database.maintenance_in_progress_title')}
                    </DialogTitle>
                    <DialogDescription>
                        {t(
                            'message.database.maintenance_in_progress_description'
                        )}
                    </DialogDescription>
                </DialogHeader>
                <IndeterminateProgress />
            </DialogContent>
        </Dialog>
    );
}
