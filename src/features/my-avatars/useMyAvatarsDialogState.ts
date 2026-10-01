import { useRef, useState } from 'react';

import type {
    MyAvatarImageCropRequest,
    MyAvatarRow,
    MyAvatarsAuthTarget
} from './myAvatarsTypes';

export function useMyAvatarsDialogState() {
    const imageUploadInputRef = useRef<HTMLInputElement | null>(null);
    const imageUploadAvatarRef = useRef<MyAvatarRow | null>(null);
    const imageUploadAuthTargetRef = useRef<MyAvatarsAuthTarget | null>(null);
    const [editDetailsAvatar, setEditDetailsAvatar] =
        useState<MyAvatarRow | null>(null);
    const [contentTagsAvatar, setContentTagsAvatar] =
        useState<MyAvatarRow | null>(null);
    const [manageTagsAvatar, setManageTagsAvatar] =
        useState<MyAvatarRow | null>(null);
    const [imageCropRequest, setImageCropRequest] =
        useState<MyAvatarImageCropRequest | null>(null);

    function clearImageUploadRequest() {
        setImageCropRequest(null);
        imageUploadAvatarRef.current = null;
        imageUploadAuthTargetRef.current = null;
    }

    return {
        clearImageUploadRequest,
        contentTagsAvatar,
        editDetailsAvatar,
        imageCropRequest,
        imageUploadAuthTargetRef,
        imageUploadAvatarRef,
        imageUploadInputRef,
        manageTagsAvatar,
        setContentTagsAvatar,
        setEditDetailsAvatar,
        setImageCropRequest,
        setManageTagsAvatar
    };
}
