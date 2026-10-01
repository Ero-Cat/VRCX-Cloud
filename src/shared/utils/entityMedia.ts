import { normalizeVrchatEndpointDomain } from '@/shared/vrchatEndpoint';

import { getColourFromUserID } from './colour';

type LooseRecord = Record<string, unknown>;

export type ImageUser = LooseRecord & {
    iconUrl?: string | null;
};

const VRCHAT_IMAGE_FILE_PATTERN =
    /(?:file\/(file_[a-f0-9-]+)\/(\d+)(?:\/file)?|image\/(file_[a-f0-9-]+)\/(\d+)\/\d+)\/?$/;

export function convertFileUrlToImageUrl(
    url: string | null | undefined,
    resolution: string | number = 128,
    endpointDomain: string | null = null
) {
    if (!url) {
        return '';
    }
    const match = url.match(VRCHAT_IMAGE_FILE_PATTERN);
    if (!match) {
        return url;
    }
    const fileId = match[1] ?? match[3];
    const version = match[2] ?? match[4];
    const endpoint = normalizeVrchatEndpointDomain(endpointDomain);
    return `${endpoint}/image/${fileId}/${version}/${resolution}`;
}

function hsvToRgb(h: number, s: number, v: number) {
    let r = 0;
    let g = 0;
    let b = 0;
    const i = Math.floor(h * 6);
    const f = h * 6 - i;
    const p = v * (1 - s);
    const q = v * (1 - f * s);
    const t = v * (1 - (1 - f) * s);

    switch (i % 6) {
        case 0:
            r = v;
            g = t;
            b = p;
            break;
        case 1:
            r = q;
            g = v;
            b = p;
            break;
        case 2:
            r = p;
            g = v;
            b = t;
            break;
        case 3:
            r = p;
            g = q;
            b = v;
            break;
        case 4:
            r = t;
            g = p;
            b = v;
            break;
        case 5:
            r = v;
            g = p;
            b = q;
            break;
        default:
            break;
    }

    const red = Math.round(r * 255);
    const green = Math.round(g * 255);
    const blue = Math.round(b * 255);
    const decColor = 0x1000000 + blue + 0x100 * green + 0x10000 * red;
    return `#${decColor.toString(16).substr(1)}`;
}

function hueToHex(hue: number, isDarkMode: boolean) {
    if (isDarkMode) {
        return hsvToRgb(hue / 65535, 0.6, 1);
    }
    return hsvToRgb(hue / 65535, 1, 0.7);
}

export function getNameColour(userId: string, isDarkMode: boolean) {
    return hueToHex(getColourFromUserID(userId), isDarkMode);
}

export function userImage(
    user: ImageUser | null | undefined,
    resolution: string | number = 128,
    endpointDomain: string | null = null
) {
    return convertFileUrlToImageUrl(user?.iconUrl, resolution, endpointDomain);
}
