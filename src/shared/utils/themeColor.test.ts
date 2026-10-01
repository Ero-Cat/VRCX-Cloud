import { describe, expect, it } from 'vitest';

import {
    normalizeCustomThemeColor,
    normalizeThemeColor,
    resolveThemeColorConfig
} from './themeColor';

describe('theme color values', () => {
    it('normalizes presets and custom hex colors', () => {
        expect(normalizeThemeColor(' BLUE ')).toBe('blue');
        expect(normalizeThemeColor('FF00AA')).toBe('#ff00aa');
        expect(normalizeThemeColor('#0af')).toBe('#00aaff');
        expect(normalizeCustomThemeColor('#12zz34')).toBeNull();
        expect(normalizeThemeColor('#12zz34')).toBe('default');
    });

    it('uses the custom accent consistently in both theme modes', () => {
        const theme = resolveThemeColorConfig('#2563eb');

        expect(theme.primary).toBe('#2563eb');
        expect(theme.primaryDark).toBe(theme.primary);
        expect(theme.ring).toBe(theme.primary);
        expect(theme.ringDark).toBe(theme.primary);
    });

    it('chooses contrasting foregrounds for dark and bright accents', () => {
        expect(resolveThemeColorConfig('#2563eb').foreground).toBe('#ffffff');
        expect(resolveThemeColorConfig('#facc15').foreground).toBe('#000000');
    });
});
