import {
    disableCommunityThemeOverrideProjection,
    saveCommunityThemeOverrideProjection
} from './installedThemes';

export async function saveCommunityThemeOverrideCss(
    cssText: string
): Promise<void> {
    await saveCommunityThemeOverrideProjection(cssText);
}

export async function clearCommunityThemeOverrideCss(): Promise<void> {
    await saveCommunityThemeOverrideCss('');
}

export async function disableCommunityThemeOverrideCss(): Promise<void> {
    await disableCommunityThemeOverrideProjection();
}

export { getCommunityThemeOverrideCssSnapshot } from './styleLayers';
