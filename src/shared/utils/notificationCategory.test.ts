import { describe, expect, it } from 'vitest';

import {
    getNotificationCategory,
    getNotificationTs
} from './notificationCategory';

describe('getNotificationCategory', () => {
    it('classifies all friend types', () => {
        const friendTypes = [
            'friendRequest',
            'ignoredFriendRequest',
            'invite',
            'requestInvite',
            'inviteResponse',
            'requestInviteResponse',
            'boop'
        ];
        for (const type of friendTypes) {
            expect(getNotificationCategory(type), type).toBe('friend');
        }
    });

    it('classifies group prefix types', () => {
        expect(getNotificationCategory('group.announcement')).toBe('group');
        expect(getNotificationCategory('group.invite')).toBe('group');
        expect(getNotificationCategory('moderation.warning')).toBe('group');
        expect(getNotificationCategory('moderation.kick')).toBe('group');
    });

    it('classifies group exact types', () => {
        expect(getNotificationCategory('groupChange')).toBe('group');
        expect(getNotificationCategory('event.announcement')).toBe('group');
    });

    it('falls back to other for unknown types', () => {
        expect(getNotificationCategory('unknown')).toBe('other');
        expect(getNotificationCategory('system.alert')).toBe('other');
        expect(getNotificationCategory('groupChange2')).toBe('other');
    });

    it('returns other for empty string', () => {
        expect(getNotificationCategory('')).toBe('other');
    });
});

describe('getNotificationTs', () => {
    it('passes through millisecond-range numbers unchanged', () => {
        expect(getNotificationTs({ created_at: 1_700_000_000_000 })).toBe(
            1_700_000_000_000
        );
    });

    it('converts second-range numbers to milliseconds', () => {
        expect(getNotificationTs({ created_at: 1_700_000 })).toBe(
            1_700_000_000
        );
    });

    it('parses ISO date strings to milliseconds', () => {
        const isoString = '2024-01-15T12:00:00.000Z';
        expect(getNotificationTs({ created_at: isoString })).toBe(
            new Date(isoString).getTime()
        );
    });

    it('prefers created_at over createdAt', () => {
        expect(
            getNotificationTs({
                created_at: 1_700_000_000_000,
                createdAt: 9999
            })
        ).toBe(1_700_000_000_000);
    });

    it('falls back to createdAt when created_at is absent', () => {
        expect(getNotificationTs({ createdAt: 1_700_000_000_000 })).toBe(
            1_700_000_000_000
        );
    });

    it('returns 0 for null, undefined, and empty string', () => {
        expect(getNotificationTs({ created_at: null })).toBe(0);
        expect(getNotificationTs({ created_at: undefined })).toBe(0);
        expect(getNotificationTs({ created_at: '' })).toBe(0);
        expect(getNotificationTs({})).toBe(0);
    });

    it('returns 0 for unparseable strings', () => {
        expect(getNotificationTs({ created_at: 'not-a-date' })).toBe(0);
        expect(getNotificationTs({ created_at: 'NaN' })).toBe(0);
    });
});
