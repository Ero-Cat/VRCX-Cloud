import { useEffect, useRef, useState } from 'react';

import configRepository from '@/repositories/configRepository';

import {
    MODERATION_TYPE_FILTERS_CONFIG_KEY,
    normalizeModerationSelectedTypes,
    parseModerationSelectedTypes
} from './moderationPageState';

export function useModerationFilters() {
    const hydratedTypeFiltersRef = useRef(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedTypes, setSelectedTypes] = useState<string[]>([]);

    useEffect(() => {
        let active = true;
        configRepository
            .getString(MODERATION_TYPE_FILTERS_CONFIG_KEY, '[]')
            .then((nextTypeFilters) => {
                if (!active) {
                    return;
                }
                setSelectedTypes(parseModerationSelectedTypes(nextTypeFilters));
                hydratedTypeFiltersRef.current = true;
            })
            .catch(() => {
                hydratedTypeFiltersRef.current = true;
            });
        return () => {
            active = false;
        };
    }, []);

    useEffect(() => {
        if (!hydratedTypeFiltersRef.current) {
            return;
        }
        configRepository.setString(
            MODERATION_TYPE_FILTERS_CONFIG_KEY,
            JSON.stringify(selectedTypes)
        );
    }, [selectedTypes]);

    return {
        normalizeSelectedTypes: normalizeModerationSelectedTypes,
        searchQuery,
        selectedTypes,
        setSearchQuery,
        setSelectedTypes
    };
}
