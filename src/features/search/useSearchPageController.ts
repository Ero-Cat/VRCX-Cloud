import { useSearchConfig } from './useSearchConfig';
import { useSearchFilters } from './useSearchFilters';
import { useSearchResults } from './useSearchResults';

export function useSearchPageController() {
    const filters = useSearchFilters();
    const config = useSearchConfig();
    const results = useSearchResults({
        ...filters,
        avatarProviderEnabled: config.avatarProviderEnabled,
        selectedAvatarProvider: config.selectedAvatarProvider,
        worldCategories: config.worldCategories
    });

    return {
        config,
        filters,
        results
    };
}
