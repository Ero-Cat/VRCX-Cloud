export const VRCHAT_WEB_BASE = 'https://vrchat.com';

export interface VrchatLaunchUrlInput {
    worldId: string;
    instanceId?: string;
    shortName?: string;
}

function homeUrl(path: string): string {
    return `${VRCHAT_WEB_BASE}/home/${path}`;
}

function pathSegment(value: string): string {
    return encodeURIComponent(value);
}

function queryParam(name: string, value: string): string {
    return `${name}=${encodeURIComponent(value)}`;
}

export function vrchatWorldUrl(worldId: string): string {
    return homeUrl(`world/${pathSegment(worldId)}`);
}

export function vrchatUserUrl(userId: string): string {
    return homeUrl(`user/${pathSegment(userId)}`);
}

export function vrchatAvatarUrl(avatarId: string): string {
    return homeUrl(`avatar/${pathSegment(avatarId)}`);
}

export function vrchatGroupUrl(groupId: string): string {
    return homeUrl(`group/${pathSegment(groupId)}`);
}

export function vrchatGroupCalendarUrl(
    groupId: string,
    eventId: string
): string {
    return homeUrl(
        `group/${pathSegment(groupId)}/calendar/${pathSegment(eventId)}`
    );
}

export function vrchatLaunchUrl({
    worldId,
    instanceId = '',
    shortName = ''
}: VrchatLaunchUrlInput): string {
    const params = [queryParam('worldId', worldId)];
    if (instanceId) {
        params.push(queryParam('instanceId', instanceId));
    }
    if (shortName) {
        params.push(queryParam('shortName', shortName));
    }
    return homeUrl(`launch?${params.join('&')}`);
}

export function vrchatPasswordUrl(): string {
    return homeUrl('password');
}

export function vrchatRegisterUrl(): string {
    return `${VRCHAT_WEB_BASE}/register`;
}
