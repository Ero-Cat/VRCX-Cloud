import {
    useCallback,
    useDeferredValue,
    useEffect,
    useMemo,
    useState
} from 'react';

import {
    FEED_FILTER_TYPES,
    isFeedFilterType,
    type FeedFilterType
} from '@/repositories/feedRepository';

const EMPTY_SCOPED_USER_IDS: readonly string[] = [];

function normalizeScopedUserIds(userIds: readonly string[]): string[] {
    return [...new Set(userIds.map((userId) => userId.trim()).filter(Boolean))];
}

function normalizeFeedFilters(filters: readonly unknown[]): FeedFilterType[] {
    const nextFilters = filters.filter(isFeedFilterType);
    return [...new Set(nextFilters)];
}

export function useFeedFilters({
    routeScopedUserIds = EMPTY_SCOPED_USER_IDS
}: {
    routeScopedUserIds?: readonly string[];
} = {}) {
    const [searchDraft, setSearchDraft] = useState('');
    const [searchQuery, setSearchQuery] = useState('');
    const [dateFrom, setDateFrom] = useState('');
    const [dateTo, setDateTo] = useState('');
    const [activeFilters, setActiveFilters] = useState<FeedFilterType[]>([]);
    const [favoritesOnly, setFavoritesOnly] = useState(false);
    const normalizedRouteScopedUserIds = useMemo(
        () => normalizeScopedUserIds(routeScopedUserIds),
        [routeScopedUserIds]
    );
    const [scopedUserIds, setScopedUserIds] = useState<string[]>(
        normalizedRouteScopedUserIds
    );
    const deferredSearchQuery = useDeferredValue(searchQuery);
    const deferredScopedUserIds = useDeferredValue(scopedUserIds);

    const setUserScope = useCallback((nextUserIds: readonly string[]) => {
        const normalized = normalizeScopedUserIds(nextUserIds);
        setScopedUserIds((current) =>
            current.length === normalized.length &&
            current.every((userId, index) => userId === normalized[index])
                ? current
                : normalized
        );
        if (normalized.length) {
            setFavoritesOnly(false);
        }
    }, []);

    useEffect(() => {
        setUserScope(normalizedRouteScopedUserIds);
    }, [normalizedRouteScopedUserIds, setUserScope]);

    const setFeedFilters = useCallback((nextFilters: FeedFilterType[]) => {
        const nextUniqueFilters = normalizeFeedFilters(nextFilters);
        setActiveFilters(
            nextUniqueFilters.length === FEED_FILTER_TYPES.length
                ? []
                : nextUniqueFilters
        );
    }, []);

    const toggleFeedFilter = useCallback((filter: FeedFilterType) => {
        setActiveFilters((current) => {
            const nextFilters = current.includes(filter)
                ? current.filter((entry) => entry !== filter)
                : [...current, filter];
            return nextFilters.length === FEED_FILTER_TYPES.length
                ? []
                : nextFilters;
        });
    }, []);

    const commitSearch = useCallback(
        (nextValue: string = searchDraft) => {
            setSearchQuery(nextValue);
        },
        [searchDraft]
    );

    const clearSearch = useCallback(() => {
        setSearchDraft('');
        setSearchQuery('');
    }, []);

    const setDateRange = useCallback((from: string, to: string) => {
        setDateFrom(from);
        setDateTo(to);
    }, []);

    return {
        activeFilters,
        deferredScopedUserIds,
        scopedUserIds,
        setUserScope,
        setDateRange,
        dateFrom,
        dateTo,
        deferredSearchQuery,
        favoritesOnly,
        feedFilterTypes: FEED_FILTER_TYPES,
        searchDraft,
        clearSearch,
        commitSearch,
        setFavoritesOnly,
        setFeedFilters,
        setSearchDraft,
        toggleFeedFilter
    };
}
