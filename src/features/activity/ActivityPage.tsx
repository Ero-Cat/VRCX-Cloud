import { lazy, Suspense } from 'react';

import { LoadingState, PageScaffold } from '@/components/layout/PageScaffold';

const ActivityPageImpl = lazy(() =>
    import('./ActivityPageImpl').then((module) => ({
        default: module.ActivityPageImpl
    }))
);

export function ActivityPage() {
    return (
        <Suspense
            fallback={
                <PageScaffold>
                    <LoadingState />
                </PageScaffold>
            }
        >
            <ActivityPageImpl />
        </Suspense>
    );
}
