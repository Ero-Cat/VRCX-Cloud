import { describe, expect, it } from 'vitest';

import { isRecord } from './record';

describe('isRecord', () => {
    it('accepts non-null objects that are not arrays', () => {
        expect(isRecord({})).toBe(true);
        expect(isRecord({ id: 'usr_123' })).toBe(true);
        expect(isRecord(new Date(0))).toBe(true);
    });

    it.each([null, undefined, false, 0, '', [], () => undefined])(
        'rejects non-record value %#',
        (value) => {
            expect(isRecord(value)).toBe(false);
        }
    );
});
