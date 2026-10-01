type PrintCleanupWarningKind = 'too_many_favorites';

type PrintFavoriteWarningLike = {
    kind?: string | null;
};

const FAVORITE_WARNING_MESSAGE_KEYS: Record<PrintCleanupWarningKind, string> = {
    too_many_favorites: 'view.tools.prints_favorites.warning.too_many_favorites'
};

const CLEANUP_WARNING_MESSAGE_KEYS: Record<PrintCleanupWarningKind, string> = {
    too_many_favorites:
        'view.tools.prints_favorites.warning.too_many_favorites_event'
};

function normalizePrintCleanupWarningKind(
    value: string | null | undefined
): PrintCleanupWarningKind | '' {
    return value === 'too_many_favorites' ? value : '';
}

export function printFavoriteWarningMessageKey(
    warning: PrintFavoriteWarningLike | null | undefined
): string {
    const kind = normalizePrintCleanupWarningKind(warning?.kind);
    return kind ? FAVORITE_WARNING_MESSAGE_KEYS[kind] : '';
}

export function printCleanupWarningMessageKey(
    kind: string | null | undefined
): string {
    const normalizedKind = normalizePrintCleanupWarningKind(kind);
    return normalizedKind ? CLEANUP_WARNING_MESSAGE_KEYS[normalizedKind] : '';
}
