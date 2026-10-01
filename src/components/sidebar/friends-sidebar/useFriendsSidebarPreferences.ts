import { useEffect, useState } from 'react';

import { normalizeSocialStatusPreset } from '@/components/dialogs/user-dialog/userProfileFields';
import configRepository from '@/repositories/configRepository';

import type { StatusPreset } from './FriendsSidebarActionItems';

export type FriendsSidebarGroupKey =
    | 'favorites'
    | 'online'
    | 'active'
    | 'offline'
    | 'sameInstance';

export type FriendsSidebarOpenGroups = Record<FriendsSidebarGroupKey, boolean>;

const groupToggleKeys: Record<FriendsSidebarGroupKey, string> = {
    favorites: 'isFriendsGroupFavorites',
    online: 'isFriendsGroupOnline',
    active: 'isFriendsGroupActive',
    offline: 'isFriendsGroupOffline',
    sameInstance: 'sidebarGroupByInstanceCollapsed'
};

export function isFriendsSidebarGroupKey(
    value: string
): value is FriendsSidebarGroupKey {
    return Object.prototype.hasOwnProperty.call(groupToggleKeys, value);
}

const defaultGroupState: FriendsSidebarOpenGroups = {
    favorites: true,
    online: true,
    active: false,
    offline: true,
    sameInstance: true
};

export function useFriendsSidebarPreferences() {
    const [openGroups, setOpenGroups] = useState(defaultGroupState);
    const [statusPresets, setStatusPresets] = useState<StatusPreset[]>([]);

    useEffect(() => {
        let active = true;
        Promise.all([
            configRepository.getBool(groupToggleKeys.favorites, true),
            configRepository.getBool(groupToggleKeys.online, true),
            configRepository.getBool(groupToggleKeys.active, false),
            configRepository.getBool(groupToggleKeys.offline, true),
            configRepository.getBool(groupToggleKeys.sameInstance, false)
        ])
            .then(
                ([
                    favorites,
                    online,
                    activeFriends,
                    offline,
                    sameInstanceCollapsed
                ]: boolean[]) => {
                    if (!active) {
                        return;
                    }
                    setOpenGroups({
                        favorites: Boolean(favorites),
                        online: Boolean(online),
                        active: Boolean(activeFriends),
                        offline: Boolean(offline),
                        sameInstance: !sameInstanceCollapsed
                    });
                }
            )
            .catch(() => {});
        return () => {
            active = false;
        };
    }, []);

    useEffect(() => {
        let active = true;
        configRepository
            .getArray<unknown>('VRCX_statusPresets', [])
            .then((nextPresets) => {
                if (active) {
                    setStatusPresets(
                        (nextPresets ?? []).map(normalizeSocialStatusPreset)
                    );
                }
            })
            .catch(() => {
                if (active) {
                    setStatusPresets([]);
                }
            });
        return () => {
            active = false;
        };
    }, []);

    function toggleSection(id: FriendsSidebarGroupKey) {
        setOpenGroups((current) => {
            const next = {
                ...current,
                [id]: !current[id]
            };
            const configKey = groupToggleKeys[id];
            if (configKey) {
                configRepository.setBool(
                    configKey,
                    id === 'sameInstance' ? !next[id] : next[id]
                );
            }
            return next;
        });
    }

    return { openGroups, statusPresets, toggleSection };
}
