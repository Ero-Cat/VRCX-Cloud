import { useEffect, useMemo, useState } from 'react';

import type { ScreenshotLibraryImage } from '@/platform/tauri/bindings';
import configRepository from '@/repositories/configRepository';

import {
    DEFAULT_SCREENSHOT_SEARCH_SORT,
    SCREENSHOT_METADATA_SEARCH_TYPES,
    SCREENSHOT_SEARCH_LAYOUT_CONFIG_KEY,
    sortScreenshotSearchRows,
    type ScreenshotSearchRow,
    type ScreenshotSearchSort,
    type ScreenshotMetadataSearchType
} from './screenshotMetadataValues';

export type ScreenshotSearchLayout = 'grid' | 'list';

export function useScreenshotMetadataSearch() {
    const [searchQuery, setSearchQuery] = useState('');
    const [searchType, setSearchType] = useState<
        ScreenshotMetadataSearchType['value']
    >(SCREENSHOT_METADATA_SEARCH_TYPES[0].value);
    const [searchRows, setSearchRows] = useState<ScreenshotSearchRow[]>([]);
    const [searchImages, setSearchImages] = useState<ScreenshotLibraryImage[]>(
        []
    );
    const [searchViewMode, setSearchViewMode] = useState<'detail' | 'results'>(
        'detail'
    );
    const [searchLayout, setSearchLayoutState] =
        useState<ScreenshotSearchLayout>('grid');
    const [searchSort, setSearchSort] = useState(
        DEFAULT_SCREENSHOT_SEARCH_SORT
    );
    const [selectedPath, setSelectedPath] = useState('');

    useEffect(() => {
        let active = true;
        configRepository
            .getString(SCREENSHOT_SEARCH_LAYOUT_CONFIG_KEY, 'grid')
            .then((storedLayout) => {
                if (active && storedLayout === 'list') {
                    setSearchLayoutState('list');
                }
            })
            .catch(() => {});
        return () => {
            active = false;
        };
    }, []);

    const currentSearchType =
        SCREENSHOT_METADATA_SEARCH_TYPES.find(
            (type) => type.value === searchType
        ) ?? SCREENSHOT_METADATA_SEARCH_TYPES[0];

    const sortedSearchRows = useMemo(
        () => sortScreenshotSearchRows(searchRows, searchSort),
        [searchRows, searchSort]
    );

    const sortedSearchImages = useMemo(() => {
        const imagesByPath = new Map(
            searchImages.map((image) => [image.path, image])
        );
        return sortedSearchRows.flatMap((row) => {
            const image = imagesByPath.get(row.filePath);
            return image ? [image] : [];
        });
    }, [searchImages, sortedSearchRows]);

    const searchNavigationPaths = useMemo(
        () => sortedSearchRows.map((row) => row.filePath),
        [sortedSearchRows]
    );
    const selectedPathIndex = searchNavigationPaths.indexOf(selectedPath);

    function setSearchLayout(layout: ScreenshotSearchLayout) {
        setSearchLayoutState(layout);
        configRepository
            .setString(SCREENSHOT_SEARCH_LAYOUT_CONFIG_KEY, layout)
            .catch(() => {});
    }

    function setSearchResults({
        rows,
        images
    }: {
        rows: ScreenshotSearchRow[];
        images: ScreenshotLibraryImage[];
    }) {
        setSearchRows(rows);
        setSearchImages(images);
    }

    function removeSearchPaths(paths: string[]) {
        const removedPaths = new Set(paths);
        setSearchRows((current) =>
            current.filter((row) => !removedPaths.has(row.filePath))
        );
        setSearchImages((current) =>
            current.filter((image) => !removedPaths.has(image.path))
        );
    }

    function resetSearchResults({ clearQuery = false } = {}) {
        setSearchRows([]);
        setSearchImages([]);
        setSelectedPath('');
        if (clearQuery) {
            setSearchQuery('');
        }
        setSearchViewMode('detail');
    }

    function toggleSearchSort(key: string) {
        setSearchSort((current: ScreenshotSearchSort) => {
            if (current.key === key) {
                return {
                    ...current,
                    asc: !current.asc
                };
            }

            return {
                key,
                asc: key !== 'dateTime'
            };
        });
    }

    return {
        currentSearchType,
        removeSearchPaths,
        resetSearchResults,
        searchLayout,
        searchNavigationPaths,
        searchQuery,
        searchRows,
        searchSort,
        searchType,
        searchViewMode,
        selectedPath,
        selectedPathIndex,
        setSearchLayout,
        setSearchQuery,
        setSearchResults,
        setSearchType,
        setSearchViewMode,
        setSelectedPath,
        sortedSearchImages,
        sortedSearchRows,
        toggleSearchSort
    };
}
