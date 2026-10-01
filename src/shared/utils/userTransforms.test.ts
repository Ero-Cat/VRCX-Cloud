import { describe, expect, it } from 'vitest';

import {
    computeTrustLevel,
    computeUserPlatform,
    trustRankFromTags
} from './userTransforms';

describe('computeTrustLevel', () => {
    it('ranks VRChat trust ranks from Visitor to Trusted User, driving both the label and the sort order used in friend/player lists', () => {
        expect(computeTrustLevel([], '')).toMatchObject({
            trustLevel: 'Visitor',
            trustColorKey: 'untrusted',
            trustSortNum: 1
        });
        expect(computeTrustLevel(['system_trust_veteran'], '')).toMatchObject({
            trustLevel: 'Trusted User',
            trustColorKey: 'veteran',
            trustSortNum: 5
        });
        expect(computeTrustLevel(['system_trust_known'], '')).toMatchObject({
            trustLevel: 'User',
            trustSortNum: 3
        });
    });

    it('flags a moderator/admin with a VIP color that overrides their trust-rank color, so staff always stand out in the list', () => {
        const result = computeTrustLevel(['system_trust_veteran'], 'moderator');
        expect(result.isModerator).toBe(true);
        expect(result.trustColorKey).toBe('vip');
    });

    it('flags a known troll with a distinct color even though their underlying trust rank is unaffected, warning the user before they interact', () => {
        const result = computeTrustLevel(
            ['system_trust_known', 'system_troll'],
            ''
        );
        expect(result.isTroll).toBe(true);
        expect(result.trustColorKey).toBe('troll');
        expect(result.trustLevel).toBe('User');
    });

    it('treats a confirmed troll tag as taking priority over a merely probable-troll tag', () => {
        const result = computeTrustLevel(
            ['system_troll', 'system_probable_troll'],
            ''
        );
        expect(result.isTroll).toBe(true);
        expect(result.isProbableTroll).toBe(false);
    });
});

describe('computeUserPlatform', () => {
    it('reports the platform the user is currently active on', () => {
        expect(computeUserPlatform('android', 'standalonewindows')).toBe(
            'android'
        );
    });

    it('falls back to the last known platform when the user is offline, so the friend list still shows where they last played from', () => {
        expect(computeUserPlatform('offline', 'android')).toBe('android');
    });

    it('falls back to the last known platform for a web-session presence, since "web" is not a real client platform worth displaying', () => {
        expect(computeUserPlatform('web', 'standalonewindows')).toBe(
            'standalonewindows'
        );
    });

    it('shows nothing rather than a stale guess when there is no last-known platform either', () => {
        expect(computeUserPlatform('offline', undefined)).toBe('');
    });
});

describe('trust rank table', () => {
    it('maps tags to ranks and prefers the highest present tag', () => {
        expect(trustRankFromTags([])).toBe('visitor');
        expect(trustRankFromTags(['system_trust_basic'])).toBe('newUser');
        expect(trustRankFromTags(['system_trust_known'])).toBe('user');
        expect(trustRankFromTags(['system_trust_trusted'])).toBe('knownUser');
        expect(trustRankFromTags(['system_trust_veteran'])).toBe('trustedUser');
        expect(
            trustRankFromTags(['system_trust_basic', 'system_trust_veteran'])
        ).toBe('trustedUser');
    });
});
