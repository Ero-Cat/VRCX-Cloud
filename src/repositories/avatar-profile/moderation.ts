import {
    commands,
    type VrchatAvatarModerationInput as IpcVrchatAvatarModerationInput
} from '@/platform/tauri/bindings';

import { isRecord, unwrapVrchatAvatarResponse } from './shared';
import type {
    AvatarModerationDeleteRecord,
    AvatarModerationInput,
    AvatarModerationRecord
} from './types';

export async function getAvatarModerations() {
    const response = unwrapVrchatAvatarResponse(
        await commands.appVrchatAvatarModerationsGet(),
        'auth/user/avatarmoderations'
    );
    return {
        ...response,
        json: Array.isArray(response.json) ? response.json.filter(isRecord) : []
    };
}

export async function sendAvatarModeration({
    avatarId
}: AvatarModerationInput) {
    const normalizedAvatarId = avatarId?.trim() ?? '';
    if (!normalizedAvatarId) {
        throw new Error(
            'AvatarProfileRepository.sendAvatarModeration requires an avatar id.'
        );
    }

    const input = {
        avatarId: normalizedAvatarId
    } satisfies IpcVrchatAvatarModerationInput;
    return unwrapVrchatAvatarResponse<AvatarModerationRecord>(
        await commands.appVrchatAvatarModerationSend(input),
        'auth/user/avatarmoderations'
    );
}

export async function deleteAvatarModeration({
    avatarId
}: AvatarModerationInput) {
    const normalizedAvatarId = avatarId?.trim() ?? '';
    if (!normalizedAvatarId) {
        throw new Error(
            'AvatarProfileRepository.deleteAvatarModeration requires an avatar id.'
        );
    }

    const input = {
        avatarId: normalizedAvatarId
    } satisfies IpcVrchatAvatarModerationInput;
    return unwrapVrchatAvatarResponse<AvatarModerationDeleteRecord>(
        await commands.appVrchatAvatarModerationDelete(input),
        'auth/user/avatarmoderations'
    );
}
