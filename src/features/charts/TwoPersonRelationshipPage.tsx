import { lazy, Suspense } from 'react';
import { useTranslation } from 'react-i18next';

import { Spinner } from '@/ui/shadcn/spinner';

const TwoPersonRelationshipPageImpl = lazy(() =>
    import('./TwoPersonRelationshipPageImpl').then((module) => ({
        default: module.TwoPersonRelationshipPage
    }))
);

export function TwoPersonRelationshipPage() {
    const { t } = useTranslation();

    return (
        <Suspense
            fallback={
                <div className="text-muted-foreground flex h-full min-h-0 items-center justify-center gap-2 text-sm">
                    <Spinner className="size-4" />
                    <span>{t('common.loading')}</span>
                </div>
            }
        >
            <TwoPersonRelationshipPageImpl />
        </Suspense>
    );
}
