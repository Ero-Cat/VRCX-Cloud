import { lazy, Suspense, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
    HashRouter,
    Navigate,
    Outlet,
    Route,
    Routes,
    useLocation
} from 'react-router';

import { GlobalHosts } from '@/app/GlobalHosts';
import { QuickSearchProvider } from '@/components/layout/QuickSearchProvider';
import { useGlobalKeyboardShortcuts } from '@/components/layout/useGlobalKeyboardShortcuts';
import { recordRouteEnter } from '@/services/telemetry/telemetryPageReach';
import { useNavigationCacheStore } from '@/state/navigationCacheStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useSessionStore } from '@/state/sessionStore';
import { Button } from '@/ui/shadcn/button';

import { isRememberedPageRoute } from './navigationRoute';
import { RouteErrorBoundary } from './RouteErrorBoundary';
import { protectedRoutes, publicRoutes, RouteLoadingFallback } from './routes';

function RouteErrorFallback() {
    const { t } = useTranslation();
    return (
        <div className="text-muted-foreground flex h-full min-h-0 flex-col items-center justify-center gap-3 text-sm">
            <Button
                variant="outline"
                size="sm"
                onClick={() => window.location.reload()}
            >
                {t('nativeShell.tray.rebuildUi')}
            </Button>
        </div>
    );
}

const AppShellLayout = lazy(() =>
    import('@/components/layout/AppShellLayout').then((module) => ({
        default: module.AppShellLayout
    }))
);

function RequireAuth() {
    const sessionPhase = useSessionStore((state) => state.sessionPhase);
    const isSessionReady = sessionPhase === 'ready';
    const isSessionPending =
        sessionPhase === 'authenticating' || sessionPhase === 'bootstrapping';
    const backendRuntimeReady = useRuntimeStore(
        (state) =>
            state.shell.backendRuntimeSnapshotHydrated &&
            !state.shell.backendRuntimeSessionHydrating
    );

    if (!backendRuntimeReady || isSessionPending) {
        return <RouteLoadingFallback />;
    }
    if (!isSessionReady) {
        return <Navigate to="/login" replace />;
    }

    return <Outlet />;
}

function RememberedPageRedirect() {
    const lastRoute = useNavigationCacheStore((state) => state.lastRoute);
    return (
        <Navigate
            to={isRememberedPageRoute(lastRoute) ? lastRoute : '/feed'}
            replace
        />
    );
}

function RedirectIfAuthenticated() {
    const sessionPhase = useSessionStore((state) => state.sessionPhase);
    const isSessionReady = sessionPhase === 'ready';
    const isSessionPending =
        sessionPhase === 'authenticating' || sessionPhase === 'bootstrapping';
    const backendRuntimeReady = useRuntimeStore(
        (state) =>
            state.shell.backendRuntimeSnapshotHydrated &&
            !state.shell.backendRuntimeSessionHydrating
    );

    if (!backendRuntimeReady || isSessionPending) {
        return <RouteLoadingFallback />;
    }
    if (isSessionReady) {
        return <RememberedPageRedirect />;
    }

    return <Outlet />;
}

function AppShellRoute() {
    return (
        <Suspense fallback={<RouteLoadingFallback />}>
            <AppShellLayout />
        </Suspense>
    );
}

function AppRouterContent() {
    const { pathname, search, hash } = useLocation();
    const sessionReady = useSessionStore(
        (state) => state.sessionPhase === 'ready'
    );
    useEffect(() => {
        const route = pathname + search + hash;
        if (sessionReady && isRememberedPageRoute(route)) {
            useNavigationCacheStore.getState().setLastRoute(route);
        }
    }, [pathname, search, hash, sessionReady]);
    useGlobalKeyboardShortcuts();
    useEffect(() => {
        recordRouteEnter(pathname);
    }, [pathname]);

    return (
        <QuickSearchProvider enabled={sessionReady}>
            <div
                data-vrcx-0-surface="app-root"
                className="vrcx-0-app-root flex h-screen min-h-0 w-full flex-col overflow-hidden"
            >
                <div
                    aria-hidden="true"
                    className="vrcx-0-background-image-transition-layer"
                />
                <div
                    data-vrcx-0-surface="route-host"
                    className="vrcx-0-route-host min-h-0 flex-1 overflow-hidden"
                >
                    <RouteErrorBoundary
                        resetKey={pathname}
                        fallback={<RouteErrorFallback />}
                    >
                        <Routes>
                            <Route element={<RedirectIfAuthenticated />}>
                                {publicRoutes.map((route) => (
                                    <Route
                                        key={route.path}
                                        path={route.path}
                                        element={route.element}
                                    />
                                ))}
                            </Route>

                            <Route element={<RequireAuth />}>
                                <Route element={<AppShellRoute />}>
                                    <Route
                                        index
                                        element={<RememberedPageRedirect />}
                                    />
                                    {protectedRoutes.map((route) => (
                                        <Route
                                            key={route.path}
                                            path={route.path}
                                            element={route.element}
                                        />
                                    ))}
                                    <Route
                                        path="*"
                                        element={
                                            <Navigate to="/feed" replace />
                                        }
                                    />
                                </Route>
                            </Route>
                        </Routes>
                    </RouteErrorBoundary>
                </div>
                <GlobalHosts />
            </div>
        </QuickSearchProvider>
    );
}

export function AppRouter() {
    const hydrated = useNavigationCacheStore((state) => state.hydrated);
    useEffect(() => {
        void useNavigationCacheStore.getState().hydrate();
    }, []);
    if (!hydrated) return <RouteLoadingFallback />;
    return (
        <HashRouter>
            <AppRouterContent />
        </HashRouter>
    );
}
