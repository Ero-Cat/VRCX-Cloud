import { describe, expect, it } from 'vitest';

import { computeBioDiff, hasBioDiffChanges } from './bioInlineDiff';

describe('computeBioDiff', () => {
    it('splits word-level changes for latin text', () => {
        const segments = computeBioDiff('hello world', 'hello VRChat world');
        expect(segments.filter((segment) => segment.kind === 'added')).toEqual([
            { kind: 'added', text: 'VRChat' }
        ]);
        expect(hasBioDiffChanges(segments)).toBe(true);
    });

    it('diffs CJK text at character level', () => {
        const segments = computeBioDiff('你好世界', '你好呀世界');
        expect(segments.filter((segment) => segment.kind === 'added')).toEqual([
            { kind: 'added', text: '呀' }
        ]);
    });

    it('keeps unchanged CJK runs without artificial spaces', () => {
        const segments = computeBioDiff('你好世界', '你好世界');
        expect(segments).toEqual([{ kind: 'same', text: '你好世界' }]);
        expect(hasBioDiffChanges(segments)).toBe(false);
    });

    it('handles empty sides', () => {
        expect(computeBioDiff(null, 'text')).toEqual([
            { kind: 'added', text: 'text' }
        ]);
        expect(computeBioDiff('text', undefined)).toEqual([
            { kind: 'removed', text: 'text' }
        ]);
    });
});
