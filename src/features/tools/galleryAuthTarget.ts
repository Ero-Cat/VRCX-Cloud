import { useRuntimeStore } from '@/state/runtimeStore';

import type { GalleryAuthTarget } from './galleryTypes';

export function getRuntimeAuthTarget(): GalleryAuthTarget {
    const runtimeAuth = useRuntimeStore.getState().auth;
    return {
        userId: runtimeAuth.currentUserId || '',
        endpoint: runtimeAuth.currentUserEndpoint || ''
    };
}

export function isRuntimeAuthTarget(authTarget: GalleryAuthTarget) {
    const runtimeAuth = getRuntimeAuthTarget();
    return (
        runtimeAuth.userId === authTarget.userId &&
        runtimeAuth.endpoint === authTarget.endpoint
    );
}
