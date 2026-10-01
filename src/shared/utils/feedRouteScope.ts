const FEED_USER_QUERY_KEY = 'user';

function normalizeUserIds(userIds: readonly string[]): string[] {
    return [...new Set(userIds.map((userId) => userId.trim()).filter(Boolean))];
}

export function readFeedRouteUserIds(searchParams: URLSearchParams): string[] {
    return normalizeUserIds(searchParams.getAll(FEED_USER_QUERY_KEY));
}

export function withFeedRouteUserIds(
    searchParams: URLSearchParams,
    userIds: readonly string[]
): URLSearchParams {
    const nextSearchParams = new URLSearchParams(searchParams);
    nextSearchParams.delete(FEED_USER_QUERY_KEY);
    for (const userId of normalizeUserIds(userIds)) {
        nextSearchParams.append(FEED_USER_QUERY_KEY, userId);
    }
    return nextSearchParams;
}

export function buildFeedRoute(userIds: readonly string[]): string {
    const searchParams = withFeedRouteUserIds(
        new URLSearchParams({ feedView: 'table' }),
        userIds
    );
    const query = searchParams.toString();
    return query ? `/feed?${query}` : '/feed';
}
