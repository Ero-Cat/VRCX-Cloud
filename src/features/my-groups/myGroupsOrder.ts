export function normalizeGroupOrder(
    order: readonly string[],
    groupIds: readonly string[]
): string[] {
    const validGroupIds = new Set(groupIds);
    const seen = new Set<string>();
    const normalized: string[] = [];

    for (const groupId of order) {
        if (validGroupIds.has(groupId) && !seen.has(groupId)) {
            normalized.push(groupId);
            seen.add(groupId);
        }
    }
    for (const groupId of groupIds) {
        if (!seen.has(groupId)) {
            normalized.push(groupId);
            seen.add(groupId);
        }
    }

    return normalized;
}

export function moveGroupInOrder(
    order: readonly string[],
    groupId: string,
    toIndex: number
): string[] | null {
    const fromIndex = order.indexOf(groupId);
    if (
        fromIndex === -1 ||
        toIndex < 0 ||
        toIndex >= order.length ||
        fromIndex === toIndex
    ) {
        return null;
    }
    const next = [...order];
    next.splice(fromIndex, 1);
    next.splice(toIndex, 0, groupId);
    return next;
}
