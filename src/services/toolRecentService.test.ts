import { beforeEach, describe, expect, it, vi } from 'vitest';

const configRepository = vi.hoisted(() => ({
    getString: vi.fn(),
    setString: vi.fn()
}));

vi.mock('@/repositories/configRepository', () => ({
    default: configRepository
}));

import { recordRecentToolOpen } from './toolRecentService';

describe('toolRecentService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        configRepository.setString.mockResolvedValue(null);
    });

    it('moves an opened tool to the front and keeps three known tools', async () => {
        configRepository.getString.mockResolvedValue(
            JSON.stringify([
                'inventory',
                'gallery',
                'gallery',
                'inventory',
                'group-calendar',
                'note-export'
            ])
        );

        await recordRecentToolOpen('gallery');

        expect(configRepository.setString).toHaveBeenCalledWith(
            'VRCX_toolsRecentList',
            JSON.stringify(['gallery', 'inventory', 'group-calendar'])
        );
    });

    it('ignores unknown tool keys', async () => {
        await recordRecentToolOpen('unknown-tool');

        expect(configRepository.getString).not.toHaveBeenCalled();
        expect(configRepository.setString).not.toHaveBeenCalled();
    });
});
