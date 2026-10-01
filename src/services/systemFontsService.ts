import { commands } from '@/platform/tauri/bindings';

let cache: Promise<string[]> | null = null;
let unavailableWarningConsumed = false;

export function loadSystemFonts(): Promise<string[]> {
    if (!cache) {
        cache = commands
            .appListSystemFonts()
            .then((fonts) => {
                if (!fonts.length) {
                    cache = null;
                }
                return fonts;
            })
            .catch((): string[] => {
                cache = null;
                return [];
            });
    }
    return cache;
}

export function consumeSystemFontsUnavailableWarning(fonts: readonly string[]) {
    if (fonts.length || unavailableWarningConsumed) {
        return false;
    }
    unavailableWarningConsumed = true;
    return true;
}
