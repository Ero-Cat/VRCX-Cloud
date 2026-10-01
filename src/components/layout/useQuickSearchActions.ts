import { createContext, useContext } from 'react';

export type QuickSearchActions = {
    openQuickSearch: () => void;
    openDirectAccessFromClipboard: () => void;
};

export const QuickSearchContext = createContext<QuickSearchActions | null>(
    null
);

export function useQuickSearchActions() {
    const actions = useContext(QuickSearchContext);
    if (!actions) {
        throw new Error('Quick search requires QuickSearchProvider.');
    }
    return actions;
}
