import { describe, expect, test } from 'vitest';

import { moveGroupInOrder, normalizeGroupOrder } from './myGroupsOrder';

describe('normalizeGroupOrder', () => {
    test('appends groups missing from the saved registry order', () => {
        expect(
            normalizeGroupOrder(['grp_b'], ['grp_a', 'grp_b', 'grp_c'])
        ).toEqual(['grp_b', 'grp_a', 'grp_c']);
    });

    test('removes stale and duplicate group ids', () => {
        expect(
            normalizeGroupOrder(
                ['grp_stale', 'grp_b', 'grp_b', 'grp_a'],
                ['grp_a', 'grp_b']
            )
        ).toEqual(['grp_b', 'grp_a']);
    });
});

describe('moveGroupInOrder', () => {
    test('moves a known group to the requested index', () => {
        expect(
            moveGroupInOrder(['grp_a', 'grp_b', 'grp_c'], 'grp_c', 0)
        ).toEqual(['grp_c', 'grp_a', 'grp_b']);
    });

    test('returns null without mutating when the move is not possible', () => {
        const order = ['grp_a', 'grp_b'];
        expect(moveGroupInOrder(order, 'grp_new', 0)).toBeNull();
        expect(moveGroupInOrder(order, 'grp_a', -1)).toBeNull();
        expect(moveGroupInOrder(order, 'grp_a', 2)).toBeNull();
        expect(moveGroupInOrder(order, 'grp_a', 0)).toBeNull();
        expect(order).toEqual(['grp_a', 'grp_b']);
    });
});
