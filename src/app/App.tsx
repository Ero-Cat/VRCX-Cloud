import { DevKitPanel } from '@/features/devkit/DevKitPanel';

import { AppBootstrap } from './bootstrap/AppBootstrap';
import { AppProviders } from './providers/AppProviders';
import { AppRouter } from './router';

export function App() {
    return (
        <AppProviders>
            <AppBootstrap />
            <AppRouter />
            <DevKitPanel />
        </AppProviders>
    );
}
