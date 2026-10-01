import { appToastManagers } from '@/services/toastService';
import { ToastProvider } from '@/ui/shadcn/toast';

import './AppToaster.css';

const TITLE_BAR_VIEWPORT_OFFSET = 'data-[position*=top]:top-[calc(2rem+32px)]';

export function AppToaster() {
    return (
        <>
            <ToastProvider
                position="top-center"
                toastManager={appToastManagers['top-center']}
                viewportClassName={TITLE_BAR_VIEWPORT_OFFSET}
            />
            <ToastProvider
                position="bottom-right"
                toastManager={appToastManagers['bottom-right']}
            />
            <ToastProvider
                position="bottom-center"
                toastManager={appToastManagers['bottom-center']}
            />
        </>
    );
}
