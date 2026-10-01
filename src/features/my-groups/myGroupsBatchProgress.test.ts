import { describe, expect, test } from 'vitest';

import { resolveMyGroupsBatchProgress } from './myGroupsBatchProgress';

const payload = {
    ownerUserId: 'usr_self',
    endpoint: 'api.vrchat.cloud',
    completed: 2,
    total: 5
};

function resolve(
    overrides: Partial<Parameters<typeof resolveMyGroupsBatchProgress>[0]>
) {
    return resolveMyGroupsBatchProgress({
        busy: true,
        currentAuthEndpoint: 'api.vrchat.cloud',
        currentUserId: 'usr_self',
        event: { count: 3, lastPayload: payload },
        previousEventCount: 2,
        ...overrides
    });
}

describe('resolveMyGroupsBatchProgress', () => {
    test('accepts a fresh event for the current account', () => {
        expect(resolve({})).toEqual({ current: 2, total: 5 });
    });

    test('ignores events that predate the current run', () => {
        expect(resolve({ previousEventCount: 3 })).toBeNull();
    });

    test('ignores events while no batch is running', () => {
        expect(resolve({ busy: false })).toBeNull();
    });

    test('ignores events from another account or endpoint', () => {
        expect(resolve({ currentUserId: 'usr_other' })).toBeNull();
        expect(resolve({ currentAuthEndpoint: 'localhost' })).toBeNull();
    });

    test('ignores payloads that are not batch progress', () => {
        expect(
            resolve({ event: { count: 3, lastPayload: { completed: 1 } } })
        ).toBeNull();
    });
});
