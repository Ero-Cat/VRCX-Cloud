import { describe, expect, it } from 'vitest';

import {
    authorTagsCsv,
    authorTagsFromCsv,
    normalizeTagName,
    styleName,
    tagsKey
} from './avatarDetailsModel';

describe('avatarDetailsModel tag helpers', () => {
    it('normalizes tag names with an idempotent prefix', () => {
        expect(normalizeTagName('  Author_Tag_Name  ', 'author_tag_')).toBe(
            'author_tag_name'
        );
        expect(normalizeTagName('author_tag_name', 'author_tag_')).toBe(
            'author_tag_name'
        );
        expect(normalizeTagName('   ', 'author_tag_')).toBe('');
    });

    it('round-trips author tag CSV through canonical tag values', () => {
        const tags = authorTagsFromCsv(
            'A, b, author_tag_c, AUTHOR_TAG_A, , author_tag_b'
        );

        expect(tags).toEqual(['author_tag_a', 'author_tag_b', 'author_tag_c']);
        expect(authorTagsCsv(tags)).toBe('a,b,c');
    });

    it('serializes only canonical author tags to CSV values', () => {
        expect(
            authorTagsCsv([
                'author_tag_a',
                'content_warning',
                'AUTHOR_TAG_B',
                'author_tag_c'
            ])
        ).toBe('a,c');
    });

    it('builds a stable dirty-check key without mutating tags', () => {
        const tags = ['author_tag_b', 'author_tag_a'];

        expect(tagsKey(tags)).toBe('author_tag_a\nauthor_tag_b');
        expect(tagsKey(['author_tag_a', 'author_tag_b'])).toBe(tagsKey(tags));
        expect(tags).toEqual(['author_tag_b', 'author_tag_a']);
    });
});

describe('avatarDetailsModel style helpers', () => {
    it('uses styleName, name, then id as the style label fallback', () => {
        expect(
            styleName({
                styleName: ' Primary ',
                name: 'Name',
                id: 'style_1'
            })
        ).toBe('Primary');
        expect(
            styleName({ styleName: '', name: ' Named ', id: 'style_1' })
        ).toBe('Named');
        expect(styleName({ name: '', id: ' style_1 ' })).toBe('style_1');
        expect(styleName(null)).toBe('');
    });
});
