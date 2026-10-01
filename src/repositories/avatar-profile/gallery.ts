import {
    entityQueryPolicies,
    fetchCachedData,
    queryKeys
} from '@/lib/entityQueryCache';
import { commands } from '@/platform/tauri/bindings';
import { DEFAULT_VRCHAT_API_ENDPOINT } from '@/shared/vrchatEndpoint';

import {
    avatarIdInput,
    isRecord,
    normalizeEntityId,
    unwrapVrchatAvatarResponse
} from './shared';
import type { AvatarGalleryFile } from './types';

export async function getAvatarGallery({
    avatarId,
    force = false
}: {
    avatarId: string;
    force?: boolean;
}): Promise<AvatarGalleryFile[]> {
    const normalizedAvatarId = normalizeEntityId(avatarId);
    if (!normalizedAvatarId) {
        throw new Error(
            'AvatarProfileRepository.getAvatarGallery requires an avatar id.'
        );
    }

    const rows = await fetchCachedData({
        queryKey: queryKeys.avatarGallery(
            normalizedAvatarId,
            DEFAULT_VRCHAT_API_ENDPOINT
        ),
        policy: entityQueryPolicies.avatarGallery,
        force,
        queryFn: async () => {
            const response = unwrapVrchatAvatarResponse(
                await commands.appVrchatAvatarGalleryGet(
                    avatarIdInput(normalizedAvatarId)
                ),
                'files'
            );
            const rows = Array.isArray(response.json)
                ? response.json
                : isRecord(response.json) && Array.isArray(response.json.files)
                  ? response.json.files
                  : [];
            return rows.filter(isRecord);
        }
    });
    return rows.slice().sort((a, b) => {
        if (!a?.order && !b?.order) {
            return 0;
        }
        return (Number(a?.order) || 0) - (Number(b?.order) || 0);
    });
}
