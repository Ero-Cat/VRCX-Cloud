import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { QuickSearchResult } from '../quickSearch';

const mocks = vi.hoisted(() => {
    const backing = new Map<string, string>();
    return {
        backing,
        contents: '',
        missing: true,
        readTextFile: vi.fn(async () => {
            if (mocks.missing) {
                throw new Error('file not found');
            }
            return mocks.contents;
        }),
        writeTextFile: vi.fn(async (_name: string, contents: string) => {
            backing.set('vrcx-file:quick-search-history.json', contents);
        })
    };
});

vi.stubGlobal('window', {
    localStorage: {
        getItem: (key: string) =>
            key === 'vrcx-file:quick-search-history.json' && !mocks.missing
                ? mocks.contents
                : null,
        setItem: (key: string, value: string) => {
            if (key === 'vrcx-file:quick-search-history.json') {
                mocks.contents = value;
                mocks.missing = false;
                mocks.backing.set(key, value);
            }
        },
        removeItem: () => {},
        clear: () => {}
    }
});

import {
    loadQuickSearchHistory,
    recordQuickSearchHistory,
    type QuickSearchHistoryScope
} from './quickSearchHistory';

const firstAccount: QuickSearchHistoryScope = {
    endpoint: 'https://api.example.test',
    userId: 'usr_first'
};

function result(index: number): QuickSearchResult {
    return {
        id: `wrld_${index}`,
        type: 'world',
        source: 'own-world',
        name: `World ${index}`,
        imageUrl: `https://example.test/${index}.png`,
        seedData: { id: `wrld_${index}` },
        memo: 'not persisted',
        note: 'not persisted'
    };
}

describe('quickSearchHistory', () => {
    beforeEach(() => {
        mocks.contents = '';
        mocks.missing = true;
        mocks.backing.clear();
    });

    it('keeps the five most recently opened unique entries', async () => {
        for (let index = 1; index <= 6; index += 1) {
            await recordQuickSearchHistory(firstAccount, result(index));
        }
        await recordQuickSearchHistory(firstAccount, result(3));

        const history = await loadQuickSearchHistory(firstAccount);

        expect(history.map((entry) => entry.id)).toEqual([
            'wrld_3',
            'wrld_6',
            'wrld_5',
            'wrld_4',
            'wrld_2'
        ]);
        expect(mocks.contents).not.toContain('seedData');
        expect(mocks.contents).not.toContain('not persisted');
    });

    it('separates history by endpoint and user', async () => {
        const secondAccount = {
            endpoint: firstAccount.endpoint,
            userId: 'usr_second'
        };
        await recordQuickSearchHistory(firstAccount, result(1));
        await recordQuickSearchHistory(secondAccount, result(2));

        await expect(loadQuickSearchHistory(firstAccount)).resolves.toEqual([
            {
                id: 'wrld_1',
                type: 'world',
                source: 'history',
                name: 'World 1',
                imageUrl: 'https://example.test/1.png'
            }
        ]);
        await expect(loadQuickSearchHistory(secondAccount)).resolves.toEqual([
            {
                id: 'wrld_2',
                type: 'world',
                source: 'history',
                name: 'World 2',
                imageUrl: 'https://example.test/2.png'
            }
        ]);
    });

    it('drops cached favorite record ids', async () => {
        mocks.contents = JSON.stringify({
            version: 1,
            accounts: {
                [JSON.stringify([firstAccount.endpoint, firstAccount.userId])]:
                    [
                        {
                            id: 'fvrt_wrong',
                            type: 'world',
                            name: 'Wrong favorite id'
                        },
                        {
                            id: 'wrld_valid',
                            type: 'world',
                            name: 'Valid world'
                        }
                    ]
            }
        });
        mocks.missing = false;

        const history = await loadQuickSearchHistory(firstAccount);

        expect(history.map((entry) => entry.id)).toEqual(['wrld_valid']);
    });

    it('serializes concurrent records without dropping an entry', async () => {
        await Promise.all([
            recordQuickSearchHistory(firstAccount, result(1)),
            recordQuickSearchHistory(firstAccount, result(2))
        ]);

        const history = await loadQuickSearchHistory(firstAccount);

        expect(history.map((entry) => entry.id)).toEqual(['wrld_2', 'wrld_1']);
    });

    it.each(['invalid json', '{"version":2,"accounts":{}}'])(
        'treats an unreadable cache as empty',
        async (contents) => {
            mocks.contents = contents;
            mocks.missing = false;

            await expect(loadQuickSearchHistory(firstAccount)).resolves.toEqual(
                []
            );
        }
    );
});
