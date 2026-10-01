import { matchPath, parsePath } from 'react-router';

import { protectedRoutes } from './routes';

export function isRememberedPageRoute(route: string): boolean {
    if (!route.startsWith('/') || route.startsWith('//')) return false;
    const { pathname = '' } = parsePath(route);
    return protectedRoutes.some(
        (definition) => matchPath(definition.path, pathname) !== null
    );
}
