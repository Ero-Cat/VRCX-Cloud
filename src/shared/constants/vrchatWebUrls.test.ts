import { describe, expect, it } from 'vitest';

import {
    vrchatAvatarUrl,
    vrchatGroupCalendarUrl,
    vrchatGroupUrl,
    vrchatLaunchUrl,
    vrchatPasswordUrl,
    vrchatRegisterUrl,
    vrchatUserUrl,
    vrchatWorldUrl
} from './vrchatWebUrls';

describe('vrchatWebUrls', () => {
    it('builds entity deep links from the VRChat web origin', () => {
        expect(vrchatWorldUrl('wrld_123')).toBe(
            'https://vrchat.com/home/world/wrld_123'
        );
        expect(vrchatUserUrl('usr_123')).toBe(
            'https://vrchat.com/home/user/usr_123'
        );
        expect(vrchatAvatarUrl('avtr_123')).toBe(
            'https://vrchat.com/home/avatar/avtr_123'
        );
        expect(vrchatGroupUrl('grp_123')).toBe(
            'https://vrchat.com/home/group/grp_123'
        );
    });

    it('builds group calendar and auth-adjacent links', () => {
        expect(vrchatGroupCalendarUrl('grp_123', 'evt_456')).toBe(
            'https://vrchat.com/home/group/grp_123/calendar/evt_456'
        );
        expect(vrchatPasswordUrl()).toBe('https://vrchat.com/home/password');
        expect(vrchatRegisterUrl()).toBe('https://vrchat.com/register');
    });

    it('builds launch URLs with encoded query values', () => {
        expect(
            vrchatLaunchUrl({
                worldId: 'wrld_123',
                instanceId: '12345~friends(usr_owner)',
                shortName: 'abc 123'
            })
        ).toBe(
            'https://vrchat.com/home/launch?worldId=wrld_123&instanceId=12345~friends(usr_owner)&shortName=abc%20123'
        );
    });
});
