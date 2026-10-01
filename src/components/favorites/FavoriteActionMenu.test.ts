import { describe, expect, it } from 'vitest';

import { resolveFavoriteAddType } from './FavoriteActionMenu';

describe('FavoriteActionMenu helpers', () => {
    it('keeps the VRC+ world type selected by the remote group', () => {
        expect(
            resolveFavoriteAddType(
                { type: 'vrcPlusWorld', name: 'worlds4' },
                'world'
            )
        ).toBe('vrcPlusWorld');
    });
});
