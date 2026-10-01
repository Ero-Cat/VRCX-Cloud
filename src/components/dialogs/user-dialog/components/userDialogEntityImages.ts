import {
    convertFileUrlToImageUrl,
    userImage
} from '@/services/entityMediaService';
import { isRecord } from '@/shared/utils/record';

export type UserDialogEntityKind = 'user' | 'world' | 'avatar' | 'group';

export function rowImage(row: unknown, kind: UserDialogEntityKind) {
    if (!isRecord(row)) {
        return '';
    }
    if (kind === 'user') {
        return userImage(row, 64);
    }
    const imageUrl = [
        row.thumbnailImageUrl,
        row.imageUrl,
        row.iconUrl,
        row.currentAvatarImageUrl
    ].find(
        (value): value is string =>
            typeof value === 'string' && Boolean(value.trim())
    );
    return convertFileUrlToImageUrl(imageUrl, 128);
}
