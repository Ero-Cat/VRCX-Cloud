import { describe, expect, it } from 'vitest';

import {
    buildGameLogSessionDurationDetails,
    getGameLogSessionPlayerDuration
} from './gameLogSessionDurations';

describe('gameLogSessionDurations', () => {
    it('preserves id and display-name accumulation when rows are batched', () => {
        const details = buildGameLogSessionDurationDetails([
            {
                displayName: 'Alice',
                userId: 'usr_alice',
                time: 60_000
            },
            {
                displayName: 'Renamed Alice',
                userId: 'usr_alice',
                time: 90_000
            },
            {
                displayName: 'Fallback User',
                userId: '',
                time: 20_000
            },
            {
                displayName: 'fallback user',
                userId: '',
                time: 30_000
            },
            {
                displayName: 'Ignored',
                userId: '',
                time: 0
            },
            {
                displayName: 'Ignored',
                userId: '',
                time: -1
            }
        ]);

        expect(details.maxDurationMs).toBe(150_000);
        expect(
            getGameLogSessionPlayerDuration(details.durationByKey, {
                displayName: 'Different Name',
                userId: 'usr_alice'
            })
        ).toBe(150_000);
        expect(
            getGameLogSessionPlayerDuration(details.durationByKey, {
                displayName: 'FALLBACK USER',
                userId: ''
            })
        ).toBe(50_000);
    });
});
