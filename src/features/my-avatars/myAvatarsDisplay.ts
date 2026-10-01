import type { CSSProperties } from 'react';

import { getTagColor } from '@/shared/constants/tags';
import { getPlatformInfo } from '@/shared/utils/avatarPlatform';

import type { MyAvatarRow, MyAvatarTag } from './myAvatarsTypes';

export function getMyAvatarPlatformInfo(
    avatar: Pick<MyAvatarRow, 'unityPackages'> | null | undefined
) {
    return getPlatformInfo(avatar?.unityPackages);
}

export function resolveMyAvatarPerformanceLabel(
    value: string | null | undefined
) {
    if (!value) {
        return '-';
    }

    return value;
}

export function resolveMyAvatarActionDisabled(
    avatar: Pick<MyAvatarRow, 'id'> | null | undefined,
    isUpdating: boolean
) {
    return isUpdating || !avatar?.id;
}

export const MY_AVATAR_TAG_BADGE_CLASS_NAME =
    'rounded-sm px-1 py-0 text-xs leading-tight';

export function resolveMyAvatarTagBadgeStyle(
    entry: Pick<MyAvatarTag, 'tag' | 'color'>
): CSSProperties {
    const color = entry?.color
        ? {
              bg: entry.color,
              text: entry.color.replace(/\/ [\d.]+\)$/, ')')
          }
        : getTagColor(entry?.tag || '');
    return {
        backgroundColor: color.bg,
        color: color.text
    };
}
