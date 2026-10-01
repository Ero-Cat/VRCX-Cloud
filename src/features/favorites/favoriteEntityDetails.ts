function hasNonEmptyString(value: unknown) {
    return typeof value === 'string' && value.trim().length > 0;
}

export function hasDisplayableEntityDetail(
    value: unknown
): value is Record<string, unknown> {
    if (!value || typeof value !== 'object') {
        return false;
    }

    const displayFields = [
        'name',
        'authorName',
        'thumbnailImageUrl',
        'imageUrl',
        'description',
        'releaseStatus'
    ];
    if (
        displayFields.some((field) =>
            hasNonEmptyString(Reflect.get(value, field))
        )
    ) {
        return true;
    }

    const tags = Reflect.get(value, 'tags');
    return Array.isArray(tags) && tags.length > 0;
}
