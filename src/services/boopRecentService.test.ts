import { beforeEach, describe, expect, it, vi } from 'vitest';

const configRepository = vi.hoisted(() => ({
    getString: vi.fn(),
    setString: vi.fn()
}));

vi.mock('@/repositories/configRepository', () => ({
    default: configRepository
}));

import {
    getRecentBoopEmojis,
    recordRecentBoopEmoji
} from './boopRecentService';

const miku = {
    kind: 'inventory' as const,
    id: 'inv_miku',
    imageUrl: 'https://example.test/miku.png',
    name: 'Miku Wave'
};
const wave = {
    kind: 'default' as const,
    id: 'default_hand_wave',
    imageUrl: 'https://wiki-files.vrchat.com/Handwave.webp',
    name: 'Hand Wave'
};

describe('boopRecentService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        configRepository.setString.mockResolvedValue(null);
    });

    it('moves the sent emoji to the front and drops its older copy', async () => {
        configRepository.getString.mockResolvedValue(
            JSON.stringify([wave, miku])
        );

        await recordRecentBoopEmoji(miku);

        expect(configRepository.setString).toHaveBeenCalledWith(
            'VRCX_boopRecentEmojis',
            JSON.stringify([miku, wave])
        );
    });

    it('rehydrates default emojis from the catalog and skips broken entries', async () => {
        configRepository.getString.mockResolvedValue(
            JSON.stringify([
                { kind: 'default', id: 'default_hand_wave' },
                { kind: 'default', id: 'default_not_a_real_emoji' },
                { kind: 'inventory', id: 'inv_no_image' },
                { kind: 'sticker', id: 'inv_wrong_kind', imageUrl: 'x' },
                miku
            ])
        );

        await expect(getRecentBoopEmojis()).resolves.toEqual([wave, miku]);
    });

    it('keeps only the ten most recent emojis', async () => {
        const stored = Array.from({ length: 10 }, (_, index) => ({
            kind: 'inventory' as const,
            id: `inv_${index}`,
            imageUrl: `https://example.test/${index}.png`,
            name: `Item ${index}`
        }));
        configRepository.getString.mockResolvedValue(JSON.stringify(stored));

        await recordRecentBoopEmoji(miku);

        const written = JSON.parse(
            configRepository.setString.mock.calls[0][1]
        ) as Array<{ id: string }>;
        expect(written).toHaveLength(10);
        expect(written[0].id).toBe('inv_miku');
        expect(written.at(-1)?.id).toBe('inv_8');
    });

    it('returns nothing for unparseable storage', async () => {
        configRepository.getString.mockResolvedValue('{not json');

        await expect(getRecentBoopEmojis()).resolves.toEqual([]);
    });
});
