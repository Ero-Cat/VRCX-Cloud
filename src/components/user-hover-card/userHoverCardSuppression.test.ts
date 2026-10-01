import { describe, expect, it } from 'vitest';

import { getHoverOpenSuppressionDeadline } from './userHoverCardSuppression';

describe('getHoverOpenSuppressionDeadline', () => {
    it('keeps click suppression active past the pending hover open timer', () => {
        expect(getHoverOpenSuppressionDeadline(1000, 500)).toBe(1600);
    });
});
