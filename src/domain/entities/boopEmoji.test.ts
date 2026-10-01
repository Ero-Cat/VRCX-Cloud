import { describe, expect, it } from 'vitest';

import { toBoopEmojiSendParams } from './boopEmoji';

describe('toBoopEmojiSendParams', () => {
    it('sends nothing for the plain boop', () => {
        expect(toBoopEmojiSendParams(null)).toEqual({
            emojiId: '',
            inventoryItemId: ''
        });
    });

    it('routes default and file emojis to emojiId', () => {
        expect(
            toBoopEmojiSendParams({
                kind: 'default',
                id: 'default_hand_wave',
                imageUrl: 'x',
                name: 'Hand Wave'
            })
        ).toEqual({ emojiId: 'default_hand_wave', inventoryItemId: '' });
        expect(
            toBoopEmojiSendParams({
                kind: 'file',
                id: 'file_custom',
                imageUrl: 'x',
                name: ''
            })
        ).toEqual({ emojiId: 'file_custom', inventoryItemId: '' });
    });

    it('routes inventory emojis to inventoryItemId', () => {
        expect(
            toBoopEmojiSendParams({
                kind: 'inventory',
                id: 'inv_miku',
                imageUrl: 'x',
                name: 'Miku'
            })
        ).toEqual({ emojiId: '', inventoryItemId: 'inv_miku' });
    });
});
