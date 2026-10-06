import { useEffect, useMemo, useRef, useState } from 'react';

import type { SidebarFriendRecord } from '@/components/sidebar/friends-sidebar/friendsSidebarModel';
import { localGameLocation } from '@/domain/friends/presence';
import { resolveSameInstanceFriendLocation } from '@/domain/friends/sameInstanceFriends';
import { isSameInstanceLocation } from '@/domain/instances/instanceRoster';
import { useFriendLocationTimeEpoch } from '@/lib/useFriendLocationTimeEpoch';
import { useNowMs } from '@/lib/useNowMs';
import memoPersistenceRepository from '@/repositories/memoPersistenceRepository';
import userProfileRepository from '@/repositories/userProfileRepository';
import vrchatInstanceRepository from '@/repositories/vrchatInstanceRepository';
import worldProfileRepository from '@/repositories/worldProfileRepository';
import { convertFileUrlToImageUrl } from '@/services/entityMediaService';
import { normalizeString as normalizeId } from '@/shared/utils/string';
import { useFriendLocationTimeStore } from '@/state/friendLocationTimeStore';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { usePreferencesStore } from '@/state/preferencesStore';
import { useRuntimeStore } from '@/state/runtimeStore';

import {
    buildUserHoverCardModel,
    normalizeInstanceCounts
} from './userHoverCardModel';

type UserHoverCardProfile = Awaited<
    ReturnType<typeof userProfileRepository.getUserProfile>
>;
type UserHoverCardPopulation = ReturnType<typeof normalizeInstanceCounts>;

type UserHoverCardDataInput = {
    userId: string;
    seed?: SidebarFriendRecord | Record<string, unknown> | null;
};

/**
 * VRChat's instance endpoint masks occupancy as 0 for instances the
 * account is not inside; count the users the roster can observe at the
 * location so the display never shows fewer people than that.
 */
function observedUserCountAtLocation(
    location: string,
    hoveredUserId: string
): number {
    if (!location) {
        return 0;
    }
    const { friendsById } = useFriendRosterStore.getState();
    let count = 0;
    let hoveredSeen = false;
    for (const friend of Object.values(friendsById)) {
        const friendId = normalizeId((friend as Record<string, unknown>)?.id);
        if (friendId && friendId === hoveredUserId) {
            hoveredSeen = true;
        }
        if (
            isSameInstanceLocation(
                resolveSameInstanceFriendLocation(friend, null),
                location
            )
        ) {
            count += 1;
        }
    }
    return count + (hoveredSeen ? 0 : 1);
}

export function useUserHoverCardData({
    userId,
    seed = null
}: UserHoverCardDataInput) {
    const endpoint = useRuntimeStore((state) => state.auth.currentUserEndpoint);
    const trustColor = usePreferencesStore((state) => state.trustColor);

    const normalizedInputUserId = normalizeId(userId);
    const shouldUseRosterSeed = !seed && Boolean(normalizedInputUserId);
    const rosterSeed = useFriendRosterStore((state) =>
        shouldUseRosterSeed
            ? (state.friendsById[normalizedInputUserId] ?? null)
            : null
    );
    const effectiveSeed = seed || rosterSeed;
    const normalizedUserId =
        normalizedInputUserId || normalizeId(effectiveSeed?.id);

    const isFriend = useFriendRosterStore((state) =>
        Boolean(normalizedUserId && state.friendsById[normalizedUserId])
    );

    const [profile, setProfile] = useState<UserHoverCardProfile | null>(null);
    const [memo, setMemo] = useState('');
    const [population, setPopulation] = useState<UserHoverCardPopulation>(null);
    const [populationLoading, setPopulationLoading] = useState(false);
    const [profileLoading, setProfileLoading] = useState(true);
    const nowMs = useNowMs();
    const [worldProfile, setWorldProfile] = useState<Awaited<
        ReturnType<typeof worldProfileRepository.getWorldProfile>
    > | null>(null);

    const localLocation = useFriendLocationTimeStore((state) =>
        normalizedUserId
            ? localGameLocation(state.byUserId[normalizedUserId])
            : ''
    );
    const model = useMemo(
        () =>
            buildUserHoverCardModel({
                seed: effectiveSeed,
                profile,
                localLocation,
                nowMs
            }),
        [effectiveSeed, localLocation, nowMs, profile]
    );
    const instanceEpoch = useFriendLocationTimeEpoch(
        normalizedUserId,
        model.location.effectiveLocation
    );

    useEffect(() => {
        let active = true;
        if (!normalizedUserId) {
            setProfileLoading(false);
            return undefined;
        }
        setProfileLoading(true);
        userProfileRepository
            .getUserProfile({
                userId: normalizedUserId,
                dialog: false,
                isFriend
            })
            .then((next: UserHoverCardProfile) => {
                if (active) {
                    setProfile(next);
                }
            })
            .catch(() => {})
            .finally(() => {
                if (active) {
                    setProfileLoading(false);
                }
            });
        memoPersistenceRepository
            .getUserMemo(normalizedUserId)
            .then((entry) => {
                if (active) {
                    setMemo(entry.memo.trim());
                }
            })
            .catch(() => {});
        return () => {
            active = false;
        };
    }, [normalizedUserId, endpoint, isFriend]);

    const worldId = model.location.worldId;
    const instanceId = model.location.instanceId;
    const isRealInstance = model.location.isRealInstance;
    const locationTagRef = useRef(model.location.effectiveLocation);
    locationTagRef.current = model.location.effectiveLocation;

    useEffect(() => {
        let active = true;
        setWorldProfile(null);
        if (!worldId) {
            return () => {
                active = false;
            };
        }
        worldProfileRepository
            .getWorldProfile({ worldId })
            .then((world) => {
                if (active) {
                    setWorldProfile(world);
                }
            })
            .catch(() => {});
        return () => {
            active = false;
        };
    }, [endpoint, worldId]);
    const worldThumb = useMemo(() => {
        const raw = worldProfile?.thumbnailImageUrl || worldProfile?.imageUrl;
        return raw ? convertFileUrlToImageUrl(raw, 512) : '';
    }, [worldProfile]);

    useEffect(() => {
        let active = true;
        setPopulation(null);
        if (!isRealInstance || !worldId || !instanceId) {
            setPopulationLoading(false);
            return undefined;
        }
        // Read via refs: `model` re-computes every second (nowMs) and must
        // stay out of the dependency array.
        const locationTag = locationTagRef.current;
        const hoveredUserId = normalizedUserId;
        setPopulationLoading(true);
        vrchatInstanceRepository
            .getInstance({ worldId, instanceId })
            .then((response) => {
                if (active) {
                    const counts = normalizeInstanceCounts(response.json);
                    const observed = observedUserCountAtLocation(
                        locationTag,
                        hoveredUserId
                    );
                    setPopulation(
                        counts
                            ? {
                                  ...counts,
                                  nUsers: Math.max(counts.nUsers, observed)
                              }
                            : counts
                    );
                }
            })
            .catch(() => {})
            .finally(() => {
                if (active) {
                    setPopulationLoading(false);
                }
            });
        return () => {
            active = false;
        };
    }, [worldId, instanceId, isRealInstance, endpoint, normalizedUserId]);

    return {
        model,
        worldThumb,
        population,
        populationLoading,
        memo,
        trustColor,
        instanceEpoch: model.variant === 'in-instance' ? instanceEpoch : 0,
        loading: profileLoading && !profile
    };
}
