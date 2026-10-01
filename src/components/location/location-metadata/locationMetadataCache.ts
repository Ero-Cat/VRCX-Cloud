import type { GroupInstanceRecord } from '@/domain/entities/group';
import { parseLocation } from '@/shared/utils/location';
import { isRecord } from '@/shared/utils/record';
import { normalizeString } from '@/shared/utils/string';

import type { LocationCacheRecord } from './locationMetadataTypes';

function recordValue(value: unknown): Record<string, unknown> | null {
    return isRecord(value) ? value : null;
}

function cacheRecord(value: unknown): LocationCacheRecord | null {
    return isRecord(value) ? value : null;
}

function instanceLocation(instance: LocationCacheRecord | null | undefined) {
    const location = recordValue(instance?.$location);
    return normalizeString(
        instance?.location || instance?.tag || location?.tag
    );
}

function locationCacheKey(location: unknown) {
    const parsed = parseLocation(location);
    if (!parsed.worldId || !parsed.instanceId) {
        return '';
    }
    return `${parsed.worldId}:${parsed.instanceId}`;
}

export function buildCachedInstanceMap(
    instances: readonly GroupInstanceRecord[]
) {
    const map = new Map<string, LocationCacheRecord>();
    for (const value of instances) {
        const instance = cacheRecord(value);
        if (!instance) {
            continue;
        }
        const location = instanceLocation(instance);
        if (location) {
            map.set(location, instance);
            const key = locationCacheKey(location);
            if (key) {
                map.set(key, instance);
            }
        }
    }
    return map;
}

export function findCachedInstance(
    cachedInstances: Map<string, LocationCacheRecord> | null | undefined,
    candidates: readonly unknown[]
) {
    if (!cachedInstances) {
        return null;
    }
    for (const candidate of candidates) {
        const location = normalizeString(candidate);
        if (!location) {
            continue;
        }
        const direct = cachedInstances.get(location);
        if (direct) {
            return direct;
        }
        const key = locationCacheKey(location);
        if (key) {
            const keyed = cachedInstances.get(key);
            if (keyed) {
                return keyed;
            }
        }
    }
    return null;
}

export function readInstanceDisplayName(instance: LocationCacheRecord | null) {
    const location = recordValue(instance?.$location);
    return normalizeString(
        instance?.displayName ||
            instance?.name ||
            instance?.instanceDisplayName ||
            location?.displayName
    );
}

export function readInstanceWorldName(instance: LocationCacheRecord | null) {
    const world = recordValue(instance?.world);
    const ref = recordValue(instance?.ref);
    const refWorld = recordValue(ref?.world);
    const location = recordValue(instance?.$location);
    const locationWorld = recordValue(location?.world);
    return normalizeString(
        instance?.worldName ||
            instance?.world_name ||
            world?.name ||
            ref?.worldName ||
            refWorld?.name ||
            location?.worldName ||
            locationWorld?.name
    );
}

export function readInstanceGroupName(instance: LocationCacheRecord | null) {
    const group = recordValue(instance?.group);
    const ref = recordValue(instance?.ref);
    const refGroup = recordValue(ref?.group);
    const location = recordValue(instance?.$location);
    const locationGroup = recordValue(location?.group);
    return normalizeString(
        instance?.groupName ||
            instance?.group_name ||
            group?.name ||
            group?.displayName ||
            ref?.groupName ||
            refGroup?.name ||
            refGroup?.displayName ||
            location?.groupName ||
            locationGroup?.name ||
            locationGroup?.displayName
    );
}

export function isInstanceClosed(instance: LocationCacheRecord | null) {
    return Boolean(
        instance?.closedAt || instance?.closed_at || instance?.isClosed
    );
}
