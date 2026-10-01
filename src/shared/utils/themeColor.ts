import {
    DEFAULT_THEME_COLOR_KEY,
    THEME_COLOR_CONFIG,
    type ThemeColorConfig
} from '@/shared/constants/themes';

const CUSTOM_THEME_COLOR_PATTERN = /^#?([0-9a-f]{6})$/i;
const SHORT_CUSTOM_THEME_COLOR_PATTERN = /^#([0-9a-f]{3})$/i;

function resolveForeground(color: string): string {
    const red = Number.parseInt(color.slice(1, 3), 16);
    const green = Number.parseInt(color.slice(3, 5), 16);
    const blue = Number.parseInt(color.slice(5, 7), 16);
    const brightness = (red * 299 + green * 587 + blue * 114) / 1000;
    return brightness >= 128 ? '#000000' : '#ffffff';
}

export function normalizeCustomThemeColor(value: string): string | null {
    const normalized = value.trim().toLowerCase();
    const fullMatch = CUSTOM_THEME_COLOR_PATTERN.exec(normalized);
    if (fullMatch) {
        return `#${fullMatch[1]}`;
    }

    const shortMatch = SHORT_CUSTOM_THEME_COLOR_PATTERN.exec(normalized);
    if (!shortMatch) {
        return null;
    }
    const [red, green, blue] = shortMatch[1];
    return `#${red}${red}${green}${green}${blue}${blue}`;
}

export function normalizeThemeColor(value: string): string {
    const normalized = value.trim().toLowerCase();
    if (Object.prototype.hasOwnProperty.call(THEME_COLOR_CONFIG, normalized)) {
        return normalized;
    }
    return normalizeCustomThemeColor(normalized) ?? DEFAULT_THEME_COLOR_KEY;
}

export function resolveThemeColorConfig(value: string): ThemeColorConfig {
    const normalized = normalizeThemeColor(value);
    const preset = THEME_COLOR_CONFIG[normalized];
    if (preset) {
        return preset;
    }

    const foreground = resolveForeground(normalized);
    return {
        key: normalized,
        label: normalized,
        swatch: normalized,
        primary: normalized,
        primaryDark: normalized,
        foreground,
        foregroundDark: foreground,
        ring: normalized,
        ringDark: normalized
    };
}
