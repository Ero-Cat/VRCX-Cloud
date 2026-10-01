import { useEffect, type Dispatch, type SetStateAction } from 'react';

import { normalizeZoomLevel } from '@/services/themeService';

type SettingsEffectsDeps = {
    setZoomInput: Dispatch<SetStateAction<string>>;
    zoomLevel: number | null;
};

export function useSettingsEffects({
    setZoomInput,
    zoomLevel
}: SettingsEffectsDeps) {
    useEffect(() => {
        setZoomInput(String(normalizeZoomLevel(zoomLevel)));
    }, [setZoomInput, zoomLevel]);
}
