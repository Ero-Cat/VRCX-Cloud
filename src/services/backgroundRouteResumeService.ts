export const BACKGROUND_MODE_RESUME_ROUTE_STORAGE_KEY =
    'VRCX_BackgroundModeResumeRoute';

export function normalizeBackgroundResumeRoute(value: string | null): string {
    const route = value?.trim().replace(/^#/, '').trim() ?? '';
    if (
        !route ||
        route === '/' ||
        route.startsWith('/login') ||
        !route.startsWith('/') ||
        route.startsWith('//') ||
        route.includes('\\') ||
        Array.from(route).some((char) => {
            const code = char.charCodeAt(0);
            return code <= 0x1f || code === 0x7f;
        })
    ) {
        return '';
    }
    return route.slice(0, 2048);
}
