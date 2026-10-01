import { useMemo, useRef, useState } from 'react';

import { computeSelectionRangeKeys } from '@/lib/useTileSelectionState';

export function useScreenshotBrowseSelection(keys: readonly string[]) {
    const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
    const lastSelectedKeyRef = useRef<string | null>(null);
    const selectedKeysSet = useMemo(
        () => new Set(selectedPaths),
        [selectedPaths]
    );
    const hasSelection = selectedKeysSet.size > 0;
    const isAllSelected =
        keys.length > 0 && keys.every((key) => selectedKeysSet.has(key));

    function clearSelection() {
        setSelectedPaths([]);
        lastSelectedKeyRef.current = null;
    }

    function removePaths(paths: string[]) {
        const removedPaths = new Set(paths);
        setSelectedPaths((current) =>
            current.filter((path) => !removedPaths.has(path))
        );
    }

    function toggleSelectAll() {
        if (isAllSelected) {
            const openFolderKeys = new Set(keys);
            setSelectedPaths((current) =>
                current.filter((path) => !openFolderKeys.has(path))
            );
            lastSelectedKeyRef.current = null;
            return;
        }
        setSelectedPaths((current) => {
            const nextPaths = new Set(current);
            for (const key of keys) {
                nextPaths.add(key);
            }
            return Array.from(nextPaths);
        });
        lastSelectedKeyRef.current = keys[keys.length - 1] ?? null;
    }

    function selectItem(
        key: string,
        checked: boolean,
        options?: { shift?: boolean }
    ) {
        const index = keys.indexOf(key);
        if (index < 0) {
            return;
        }
        const lastKey = lastSelectedKeyRef.current;
        const lastIndex =
            options?.shift && lastKey !== null ? keys.indexOf(lastKey) : -1;
        const rangeKeys =
            lastIndex >= 0
                ? computeSelectionRangeKeys({
                      keys,
                      fromIndex: lastIndex,
                      toIndex: index
                  })
                : [key];
        setSelectedPaths((current) => {
            const nextPaths = new Set(current);
            for (const rangeKey of rangeKeys) {
                if (checked) {
                    nextPaths.add(rangeKey);
                } else {
                    nextPaths.delete(rangeKey);
                }
            }
            return Array.from(nextPaths);
        });
        lastSelectedKeyRef.current = key;
    }

    return {
        clearSelection,
        hasSelection,
        isAllSelected,
        removePaths,
        selectedKeysSet,
        selectedPaths,
        selectItem,
        toggleSelectAll
    };
}
