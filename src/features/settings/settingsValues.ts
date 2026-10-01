export {
    DEFAULT_HMD_NOTIFICATION_ACTIVITY_FILTERS,
    DEFAULT_OVERLAY_ACTIVITY_FILTERS,
    DEFAULT_TTS_NOTIFICATION_ACTIVITY_FILTERS,
    DEFAULT_VR_NOTIFICATION_ACTIVITY_FILTERS,
    DEFAULT_WEBHOOK_ACTIVITY_FILTERS,
    HMD_DEFAULT_SCOPES,
    OVERLAY_ACTIVITY_TYPE_DEFINITIONS,
    normalizeOverlayActivityFilters,
    overlayActivityTypeLabelKey
} from '@/shared/constants/overlayActivityFilters';

const TABLE_PAGE_SIZE_SUGGESTIONS = [
    5, 10, 15, 20, 25, 30, 50, 75, 100, 150, 200, 250, 500, 1000
];
export const TABLE_PAGE_SIZE_DEFAULTS = [10, 15, 20, 25, 50, 100];

export function normalizeTablePageSizes(input: unknown): number[] {
    const source = Array.isArray(input) ? input : TABLE_PAGE_SIZE_DEFAULTS;
    const values = source
        .map((value) => Number.parseInt(String(value), 10))
        .filter(
            (value) => Number.isFinite(value) && value > 0 && value <= 1000
        );
    const uniqueSorted = Array.from(new Set(values)).sort(
        (left, right) => left - right
    );
    return uniqueSorted.length ? uniqueSorted : [...TABLE_PAGE_SIZE_DEFAULTS];
}

export function buildTablePageSizeOptions(
    draftSizes: readonly (string | number)[] | null | undefined
) {
    return normalizeTablePageSizes([
        ...TABLE_PAGE_SIZE_SUGGESTIONS,
        ...(Array.isArray(draftSizes) ? draftSizes : [])
    ]);
}

export function filterTablePageSizeOptions(
    options: readonly number[] | null | undefined,
    query: string
) {
    const searchTerm = query.trim();
    if (!searchTerm) {
        return Array.isArray(options) ? options : [];
    }
    return (options ?? []).filter((size) => String(size).includes(searchTerm));
}

export function parseIntegerInput(value: string | number, fallback: number) {
    const parsed = Number.parseInt(String(value), 10);
    return Number.isFinite(parsed) ? parsed : fallback;
}
