import { describe, expect, it } from 'vitest';

import { buildFeedColumnFavoriteIds } from './feedColumnScope';
import type { FeedColumnConfig } from './feedColumnsState';

function createColumn(groupKeys: 'all' | string[]): FeedColumnConfig {
    return {
        id: 'test',
        title: 'Test',
        width: 320,
        friendScope: { kind: 'favorites', groupKeys },
        feedTypes: ['GPS']
    };
}

describe('feed column scope helpers', () => {
    it('keeps local favorite groups separate from remote groups with the same key', () => {
        const ids = buildFeedColumnFavoriteIds({
            column: createColumn(['local:shared']),
            localFriendFavorites: {
                shared: ['usr_local']
            },
            remoteFavoritesById: {
                fav_remote: {
                    type: 'friend',
                    favoriteId: 'usr_remote',
                    $groupKey: 'shared'
                }
            }
        });

        expect([...ids]).toEqual(['usr_local']);
    });

    it('does not match local groups when a remote group is selected', () => {
        const ids = buildFeedColumnFavoriteIds({
            column: createColumn(['shared']),
            localFriendFavorites: {
                shared: ['usr_local']
            },
            remoteFavoritesById: {
                fav_remote: {
                    type: 'friend',
                    favoriteId: 'usr_remote',
                    $groupKey: 'shared'
                }
            }
        });

        expect([...ids]).toEqual(['usr_remote']);
    });
});
