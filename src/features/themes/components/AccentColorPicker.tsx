import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { THEME_COLORS } from '@/shared/constants/themes';
import { normalizeCustomThemeColor } from '@/shared/utils/themeColor';
import { Button } from '@/ui/shadcn/button';
import { Input } from '@/ui/shadcn/input';

import { themeColorLabel } from '../themeHelpers';
import type { useThemesController } from '../useThemesController';

type AccentColorPickerProps = Pick<
    ReturnType<typeof useThemesController>,
    'accentControlled' | 'themeColor' | 'updateThemeColor'
>;

const DEFAULT_CUSTOM_THEME_COLOR = '#2563eb';

export function AccentColorPicker({
    accentControlled,
    themeColor,
    updateThemeColor
}: AccentColorPickerProps) {
    const { t } = useTranslation();
    const activeCustomColor = normalizeCustomThemeColor(themeColor);
    const [customColorDraft, setCustomColorDraft] = useState(
        activeCustomColor ?? DEFAULT_CUSTOM_THEME_COLOR
    );
    const customColorActive = activeCustomColor !== null;
    const normalizedCustomColor = normalizeCustomThemeColor(customColorDraft);
    const customColorInvalid = normalizedCustomColor === null;
    const customColorValue =
        normalizedCustomColor ??
        activeCustomColor ??
        DEFAULT_CUSTOM_THEME_COLOR;

    useEffect(() => {
        if (activeCustomColor) {
            setCustomColorDraft(activeCustomColor);
        }
    }, [activeCustomColor]);

    function commitCustomColor(value: string): void {
        const normalized = normalizeCustomThemeColor(value);
        if (!normalized) {
            setCustomColorDraft(
                activeCustomColor ?? DEFAULT_CUSTOM_THEME_COLOR
            );
            return;
        }
        setCustomColorDraft(normalized);
        void updateThemeColor(normalized);
    }

    return (
        <div className="border-border/70 bg-card/60 flex min-w-0 flex-col gap-2 rounded-lg border px-3 py-2.5">
            <div className="flex min-w-0 flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
                <div className="text-sm font-medium">
                    {t('view.themes.accent.header')}
                </div>
                {accentControlled ? (
                    <p className="text-muted-foreground text-xs">
                        {t('view.community_themes.installed.accent_controlled')}
                    </p>
                ) : null}
            </div>
            <div className="flex flex-wrap items-center gap-1.5">
                {THEME_COLORS.map((color) => (
                    <Button
                        key={color.key}
                        type="button"
                        size="sm"
                        variant={
                            themeColor === color.key ? 'default' : 'outline'
                        }
                        className="h-7"
                        disabled={accentControlled}
                        onClick={() => updateThemeColor(color.key)}
                    >
                        <span
                            aria-hidden="true"
                            className="border-foreground/10 size-2.5 shrink-0 rounded-full border"
                            style={{
                                backgroundColor: color.swatch
                            }}
                        />
                        {themeColorLabel(color, t)}
                    </Button>
                ))}
                <Button
                    type="button"
                    size="sm"
                    variant={customColorActive ? 'default' : 'outline'}
                    className="h-7"
                    aria-pressed={customColorActive}
                    disabled={accentControlled || customColorInvalid}
                    onClick={() => commitCustomColor(customColorValue)}
                >
                    <span
                        aria-hidden="true"
                        className="border-foreground/10 size-2.5 shrink-0 rounded-full border"
                        style={{ backgroundColor: customColorValue }}
                    />
                    {t('view.themes.accent.custom')}
                </Button>
                <Input
                    type="color"
                    className="size-7 shrink-0 cursor-pointer p-1"
                    aria-label={t('view.themes.accent.custom')}
                    disabled={accentControlled}
                    value={customColorValue}
                    onChange={(event) => commitCustomColor(event.target.value)}
                />
                <Input
                    className="h-7 w-28 font-mono uppercase"
                    aria-label={t('view.themes.accent.hex_label')}
                    aria-invalid={customColorInvalid || undefined}
                    disabled={accentControlled}
                    inputMode="text"
                    maxLength={7}
                    spellCheck={false}
                    value={customColorDraft}
                    onChange={(event) =>
                        setCustomColorDraft(event.target.value)
                    }
                    onBlur={(event) => commitCustomColor(event.target.value)}
                    onKeyDown={(event) => {
                        if (event.key === 'Enter') {
                            event.currentTarget.blur();
                        }
                    }}
                />
            </div>
        </div>
    );
}
