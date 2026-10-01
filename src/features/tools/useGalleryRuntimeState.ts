import { useQuery } from '@tanstack/react-query';

import { queryKeys } from '@/lib/entityQueryCache';
import userProfileRepository from '@/repositories/userProfileRepository';
import { useModalStore } from '@/state/modalStore';
import { useRuntimeStore } from '@/state/runtimeStore';

export function useGalleryRuntimeState() {
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const currentEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    const currentUserSnapshot = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const openImagePreview = useModalStore((state) => state.openImagePreview);
    const mediaProfileQuery = useQuery({
        queryKey: [
            ...queryKeys.userAppearanceProfile(
                currentUserId || '',
                currentEndpoint
            ),
            'self'
        ],
        queryFn: () =>
            userProfileRepository.getUserAppearanceProfile({
                userId: currentUserId || '',
                asSelf: true
            }),
        enabled: Boolean(currentUserId),
        staleTime: 0,
        gcTime: 0,
        retry: false,
        refetchOnWindowFocus: false
    });
    const mediaProfile = mediaProfileQuery.data ?? null;
    const bannerCustomUrl = mediaProfile?.bannerCustomUrl || '';
    const userIcon = mediaProfile?.userIcon || '';
    const isVrcPlusSupporter = Boolean(
        currentUserSnapshot?.$isVRCPlus ||
        currentUserSnapshot?.tags?.includes?.('system_supporter') ||
        globalThis.$debug?.debugVrcPlus
    );

    return {
        currentEndpoint,
        currentUserId,
        isVrcPlusSupporter,
        openImagePreview,
        bannerCustomUrl,
        mediaProfile,
        mediaProfileLoading:
            Boolean(currentUserId) && mediaProfileQuery.isPending,
        mediaProfileError: mediaProfileQuery.error?.message || '',
        refreshMediaProfile: async () => {
            const result = await mediaProfileQuery.refetch({
                throwOnError: true
            });
            return result.data ?? null;
        },
        userIcon
    };
}
