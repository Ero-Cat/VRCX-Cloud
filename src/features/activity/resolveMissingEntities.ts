export async function resolveMissingEntities<T>({
    ids,
    isActive,
    fetchOne,
    onResolved
}: {
    ids: string[];
    isActive: () => boolean;
    fetchOne: (id: string) => Promise<T | null>;
    onResolved: (id: string, value: T) => void;
}): Promise<void> {
    await Promise.all(
        ids.map(async (id) => {
            try {
                const value = await fetchOne(id);
                if (value && isActive()) {
                    onResolved(id, value);
                }
            } catch {
                return;
            }
        })
    );
}
