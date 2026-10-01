import { useEffect } from 'react';

import { isEditableTarget } from '@/components/layout/useGlobalKeyboardShortcuts';

export function useClearSelectionOnEscape(
    hasSelection: boolean,
    clearSelection: () => void
) {
    useEffect(() => {
        if (!hasSelection) {
            return undefined;
        }
        function handleKeyDown(event: KeyboardEvent) {
            if (event.key !== 'Escape' || isEditableTarget(event.target)) {
                return;
            }
            clearSelection();
        }
        window.addEventListener('keydown', handleKeyDown);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, [clearSelection, hasSelection]);
}
