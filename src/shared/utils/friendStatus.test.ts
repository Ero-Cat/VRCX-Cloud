import { describe, expect, it } from 'vitest';

import {
    normalizeUserStatus,
    sortStatus,
    userStatusFromValue
} from './friendStatus';

describe('normalizeUserStatus', () => {
    it('accepts both spaced and spaceless status aliases', () => {
        expect(normalizeUserStatus('joinme')).toBe('join me');
        expect(normalizeUserStatus('JoinMe')).toBe('join me');
        expect(normalizeUserStatus('  join me  ')).toBe('join me');
        expect(normalizeUserStatus('askme')).toBe('ask me');
        expect(normalizeUserStatus('Ask Me')).toBe('ask me');
    });

    it('collapses offline sentinels', () => {
        expect(normalizeUserStatus('offline:offline')).toBe('offline');
        expect(normalizeUserStatus('offline 2 hours ago')).toBe('offline');
    });

    it('passes through other values lowercased', () => {
        expect(normalizeUserStatus('Active')).toBe('active');
        expect(normalizeUserStatus('busy')).toBe('busy');
        expect(normalizeUserStatus(null)).toBe('');
    });
});

describe('sortStatus', () => {
    it('returns 0 for identical statuses', () => {
        expect(sortStatus('join me', 'join me')).toBe(0);
        expect(sortStatus('active', 'active')).toBe(0);
        expect(sortStatus('ask me', 'ask me')).toBe(0);
        expect(sortStatus('busy', 'busy')).toBe(0);
        expect(sortStatus('offline', 'offline')).toBe(0);
    });

    it('orders join me before all others', () => {
        expect(sortStatus('join me', 'active')).toBeLessThan(0);
        expect(sortStatus('join me', 'ask me')).toBeLessThan(0);
        expect(sortStatus('join me', 'busy')).toBeLessThan(0);
        expect(sortStatus('join me', 'offline')).toBeLessThan(0);
    });

    it('orders active before ask me, busy, offline', () => {
        expect(sortStatus('active', 'ask me')).toBeLessThan(0);
        expect(sortStatus('active', 'busy')).toBeLessThan(0);
        expect(sortStatus('active', 'offline')).toBeLessThan(0);
    });

    it('orders ask me before busy and offline', () => {
        expect(sortStatus('ask me', 'busy')).toBeLessThan(0);
        expect(sortStatus('ask me', 'offline')).toBeLessThan(0);
    });

    it('orders busy before offline', () => {
        expect(sortStatus('busy', 'offline')).toBeLessThan(0);
    });

    it('orders offline after all others', () => {
        expect(sortStatus('offline', 'join me')).toBeGreaterThan(0);
        expect(sortStatus('offline', 'active')).toBeGreaterThan(0);
        expect(sortStatus('offline', 'ask me')).toBeGreaterThan(0);
        expect(sortStatus('offline', 'busy')).toBeGreaterThan(0);
    });

    it('is antisymmetric for all ordered pairs', () => {
        const ordered: [string, string][] = [
            ['join me', 'active'],
            ['join me', 'ask me'],
            ['join me', 'busy'],
            ['join me', 'offline'],
            ['active', 'ask me'],
            ['active', 'busy'],
            ['active', 'offline'],
            ['ask me', 'busy'],
            ['ask me', 'offline'],
            ['busy', 'offline']
        ];
        for (const [higher, lower] of ordered) {
            expect(Math.sign(sortStatus(higher, lower))).toBe(
                -Math.sign(sortStatus(lower, higher))
            );
        }
    });

    it('returns 0 for unknown statuses', () => {
        expect(sortStatus('unknown', 'other')).toBe(0);
    });
});

describe('userStatusFromValue', () => {
    it('resolves the five known statuses, including spaceless aliases', () => {
        expect(userStatusFromValue('active')).toBe('active');
        expect(userStatusFromValue('joinme')).toBe('join me');
        expect(userStatusFromValue('Ask Me')).toBe('ask me');
        expect(userStatusFromValue('busy')).toBe('busy');
        expect(userStatusFromValue('offline:offline')).toBe('offline');
    });

    it('returns an empty string for anything outside the known set', () => {
        expect(userStatusFromValue('sleeping')).toBe('');
        expect(userStatusFromValue(null)).toBe('');
        expect(userStatusFromValue(undefined)).toBe('');
    });
});
