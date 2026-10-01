import { useTranslation } from 'react-i18next';

import { ToolbarSegmented } from '@/components/layout/ToolbarControls';
import { Badge } from '@/ui/shadcn/badge';
import {
    ToggleGroup,
    ToggleGroupItem,
    ToggleGroupSeparator
} from '@/ui/shadcn/toggle-group';

import { THEME_MODE_OPTIONS, themeModeLabel } from '../themeHelpers';
import type { useThemesController } from '../useThemesController';

type ThemeSourceSelectorProps = Pick<
    ReturnType<typeof useThemesController>,
    | 'customCssBadge'
    | 'visibleSource'
    | 'selectBuiltInSource'
    | 'selectBackgroundSource'
    | 'selectCommunitySource'
    | 'themeMode'
    | 'updateThemeMode'
>;

export function ThemeSourceSelector({
    customCssBadge,
    visibleSource,
    selectBuiltInSource,
    selectBackgroundSource,
    selectCommunitySource,
    themeMode,
    updateThemeMode
}: ThemeSourceSelectorProps) {
    const { t } = useTranslation();

    return (
        <div className="border-border/70 bg-card/70 flex min-w-0 flex-col gap-3 rounded-lg border px-3 py-2.5">
            <div className="flex min-w-0 flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
                <div className="grid min-w-0 gap-1">
                    <div className="text-sm font-medium">
                        {t('view.themes.summary.header')}
                    </div>
                    {customCssBadge ? (
                        <div>
                            <Badge
                                variant="secondary"
                                className="h-5 rounded-md px-1.5 text-xs font-normal"
                            >
                                {customCssBadge}
                            </Badge>
                        </div>
                    ) : null}
                </div>
                <ToggleGroup
                    variant="outline"
                    size="sm"
                    value={[visibleSource]}
                >
                    <ToggleGroupItem
                        value="built-in"
                        onClick={selectBuiltInSource}
                    >
                        {t('view.themes.source.built_in')}
                    </ToggleGroupItem>
                    <ToggleGroupSeparator />
                    <ToggleGroupItem
                        value="background"
                        onClick={selectBackgroundSource}
                    >
                        {t('view.themes.source.background')}
                    </ToggleGroupItem>
                    <ToggleGroupSeparator />
                    <ToggleGroupItem
                        value="community"
                        onClick={selectCommunitySource}
                    >
                        {t('view.themes.source.community')}
                    </ToggleGroupItem>
                </ToggleGroup>
            </div>

            {visibleSource === 'built-in' ? (
                <div className="border-border/70 flex min-w-0 flex-col gap-2 border-t pt-2 sm:flex-row sm:items-center sm:justify-between">
                    <div className="text-muted-foreground text-xs">
                        {t('view.themes.source.built_in_description')}
                    </div>
                    <ToolbarSegmented
                        value={themeMode}
                        onValueChange={updateThemeMode}
                        options={THEME_MODE_OPTIONS.map((mode) => ({
                            value: mode,
                            label: themeModeLabel(mode, t)
                        }))}
                    />
                </div>
            ) : null}
        </div>
    );
}
