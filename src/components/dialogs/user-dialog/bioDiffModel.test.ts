import { describe, expect, it } from 'vitest';

import {
    collapseBioDiffContext,
    computeBioLineDiff,
    hasBioChanges
} from './bioDiffModel';

describe('computeBioLineDiff', () => {
    it('marks removed and added lines around unchanged context', () => {
        const lines = computeBioLineDiff('a\nb\nc', 'a\nx\nc');
        expect(lines).toEqual([
            { kind: 'same', text: 'a' },
            { kind: 'removed', text: 'b' },
            { kind: 'added', text: 'x' },
            { kind: 'same', text: 'c' }
        ]);
    });

    it('handles empty sides', () => {
        expect(computeBioLineDiff(null, 'hello')).toEqual([
            { kind: 'added', text: 'hello' }
        ]);
        expect(computeBioLineDiff('hello', '')).toEqual([
            { kind: 'removed', text: 'hello' }
        ]);
        expect(computeBioLineDiff(null, undefined)).toEqual([]);
    });

    it('normalizes windows line endings', () => {
        const lines = computeBioLineDiff('a\r\nb', 'a\nb');
        expect(lines.every((line) => line.kind === 'same')).toBe(true);
    });

    it('detects whether anything changed', () => {
        expect(hasBioChanges(computeBioLineDiff('a', 'a'))).toBe(false);
        expect(hasBioChanges(computeBioLineDiff('a', 'b'))).toBe(true);
    });
});

describe('collapseBioDiffContext', () => {
    it('keeps context around changes and elides the rest', () => {
        const lines = computeBioLineDiff(
            '1\n2\n3\n4\n5\n6\n7\n8\n9\n10',
            '1\n2\n3\n4\nX\n6\n7\n8\n9\n10'
        );
        const collapsed = collapseBioDiffContext(lines, 1);
        const elisions = collapsed.filter((line) => line.text === '…');
        expect(elisions.length).toBeGreaterThan(0);
        expect(collapsed.filter((line) => line.kind === 'removed')).toEqual([
            { kind: 'removed', text: '5' }
        ]);
        // Everything between the first and last elision marker is dropped.
        expect(collapsed.length).toBeLessThan(lines.length);
    });

    it('returns all lines when everything is context', () => {
        const lines = computeBioLineDiff('a\nb', 'a\nX');
        expect(collapseBioDiffContext(lines, 3)).toEqual(lines);
    });
});
