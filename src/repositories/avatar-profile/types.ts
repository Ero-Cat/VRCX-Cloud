import type { AvatarLocalTag } from '@/domain/entities/avatar';
import type {
    AvatarUpdateRequest,
    AvatarListSort,
    QueryOrder,
    ReleaseStatusFilter
} from '@/platform/tauri/bindings';

export type AvatarRecord = Record<string, unknown>;

export type AvatarStyleRecord = AvatarRecord & {
    id?: string;
    name?: string;
    styleName?: string;
};

export type AvatarGalleryFile = AvatarRecord & {
    id?: string;
    fileId?: string;
    order?: number | string;
    url?: string;
    fileUrl?: string;
    imageUrl?: string;
    versions?: Array<
        AvatarRecord & {
            file?: AvatarRecord & { url?: string };
        }
    >;
};

export type AvatarModerationRecord = AvatarRecord & {
    avatarModerationType?: string;
    created?: string | number;
    targetAvatarId?: string;
};

export type AvatarModerationDeleteRecord = AvatarRecord & {
    OK?: string;
};

export interface AvatarProfileExtras extends AvatarRecord {
    cachedAvatar?: boolean;
    localTags?: AvatarLocalTag[];
    timeSpent?: number;
    memo?: string;
}

export interface AvatarListOptions {
    userId?: string;
    user?: string;
    n?: number;
    offset?: number;
    sort?: AvatarListSort;
    order?: QueryOrder;
    releaseStatus?: ReleaseStatusFilter;
}

export interface AvatarIdInput {
    avatarId?: string;
}

export interface SaveAvatarInput extends AvatarIdInput {
    params: AvatarUpdateRequest;
}

export interface AvatarStylesInput {
    force?: boolean;
}

export interface AvatarProfileInput extends AvatarIdInput {
    force?: boolean;
    dialog?: boolean;
    allowLocalFallback?: boolean;
    currentUserId?: string | null;
}

export type AvatarModerationInput = AvatarIdInput;
