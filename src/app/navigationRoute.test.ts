import { describe, expect, it } from 'vitest';

import { isRememberedPageRoute } from './navigationRoute';

describe('remembered page routes', () => {
    it('accepts registered pages, dynamic routes, and page query state', () => {
        for (const route of [
            '/feed',
            '/settings?tab=appearance',
            '/dashboard/example',
            '/tools/group-moderation/grp_example'
        ]) {
            expect(isRememberedPageRoute(route)).toBe(true);
        }
    });

    it('excludes login, startup, unknown pages, and external destinations', () => {
        for (const route of [
            '/login',
            '/',
            '/removed-page',
            '//example.com/feed',
            'https://example.com/feed'
        ]) {
            expect(isRememberedPageRoute(route)).toBe(false);
        }
    });
});
