import { describe, expect, it } from 'vitest';

import {
    convertFileUrlToImageUrl,
    getNameColour,
    userImage
} from './entityMedia';

describe('entityMedia', () => {
    it('converts VRChat file URLs to image URLs with endpoint normalization', () => {
        expect(
            convertFileUrlToImageUrl(
                'https://api.vrchat.cloud/api/1/file/file_1234abcd-0000-1111-2222-abcdefabcdef/7/file',
                256,
                'https://api.vrchat.cloud/api/1'
            )
        ).toBe(
            'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/7/256'
        );
        expect(
            convertFileUrlToImageUrl(
                'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/7/256',
                1024,
                'https://api.vrchat.cloud/api/1'
            )
        ).toBe(
            'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/7/1024'
        );
        expect(
            convertFileUrlToImageUrl('https://images.example/avatar.png')
        ).toBe('https://images.example/avatar.png');
        expect(convertFileUrlToImageUrl(null)).toBe('');
    });

    it('keeps deterministic name colors for light and dark mode', () => {
        expect(
            getNameColour('usr_00000000-0000-0000-0000-000000000001', false)
        ).toBe('#4400b3');
        expect(
            getNameColour('usr_00000000-0000-0000-0000-000000000001', true)
        ).toBe('#a066ff');
        expect(getNameColour('', false)).toBe('#b300a1');
    });

    it('resolves user images from iconUrl at the requested resolution', () => {
        const iconUrl =
            'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/2/256';
        expect(userImage({ iconUrl }, 64)).toBe(
            'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/2/64'
        );
        expect(userImage({ iconUrl })).toBe(
            'https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/2/128'
        );
        expect(userImage({}, 64)).toBe('');
        expect(userImage(null)).toBe('');
    });
});
