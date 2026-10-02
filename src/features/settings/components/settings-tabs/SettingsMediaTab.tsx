import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';

import { usePreferencesStore } from '@/state/preferencesStore';
import {
    NumberField,
    NumberFieldDecrement,
    NumberFieldGroup,
    NumberFieldIncrement,
    NumberFieldInput
} from '@/ui/shadcn/number-field';
import { Switch } from '@/ui/shadcn/switch';

import { useSettingsPageSection } from '../../SettingsPageStateContext';
import { SettingsCard } from '../SettingsCard';
import { Field } from '../SettingsField';
import { SettingsTabContent } from '../SettingsViewParts';

export function SettingsMediaTab() {
    const media = useSettingsPageSection('media');
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            saveInstancePrints: state.saveInstancePrints,
            cropInstancePrints: state.cropInstancePrints,
            autoDeleteOldPrints: state.autoDeleteOldPrints,
            autoDeletePrintsLimit: state.autoDeletePrintsLimit,
            saveInstanceStickers: state.saveInstanceStickers,
            saveInstanceEmoji: state.saveInstanceEmoji
        }))
    );
    const {
        onSaveInstancePrintsChange,
        onCropInstancePrintsChange,
        onAutoDeleteOldPrintsChange,
        onAutoDeletePrintsLimitChange,
        onAutoDeletePrintsLimitBlur,
        onSaveInstanceStickersChange,
        onSaveInstanceEmojiChange
    } = media;
    const { t } = useTranslation();
    return (
        <SettingsTabContent value="media">
            <SettingsCard
                cardId="media.prints"
                title={t(
                    'view.settings.advanced.advanced.save_instance_prints_to_file.header'
                )}
                description={t(
                    'view.settings.advanced.advanced.save_instance_prints_to_file.header_tooltip'
                )}
            >
                <Field
                    label={t(
                        'view.settings.advanced.advanced.save_instance_prints_to_file.description'
                    )}
                >
                    <Switch
                        checked={prefs.saveInstancePrints}
                        onCheckedChange={onSaveInstancePrintsChange}
                    />
                </Field>
                <Field
                    label={t(
                        'view.settings.advanced.advanced.save_instance_prints_to_file.crop'
                    )}
                >
                    <Switch
                        checked={prefs.cropInstancePrints}
                        disabled={!prefs.saveInstancePrints}
                        onCheckedChange={onCropInstancePrintsChange}
                    />
                </Field>
                <Field
                    label={t(
                        'view.settings.advanced.advanced.auto_delete_prints.enable'
                    )}
                    description={t(
                        'view.settings.advanced.advanced.auto_delete_prints.description'
                    )}
                >
                    <Switch
                        checked={prefs.autoDeleteOldPrints}
                        onCheckedChange={onAutoDeleteOldPrintsChange}
                    />
                </Field>
                <Field
                    label={t(
                        'view.settings.advanced.advanced.auto_delete_prints.limit'
                    )}
                    description={t(
                        'view.settings.advanced.advanced.auto_delete_prints.limit_description'
                    )}
                >
                    <NumberField
                        min={30}
                        max={60}
                        step={1}
                        allowOutOfRange
                        className="w-32"
                        value={prefs.autoDeletePrintsLimit ?? 60}
                        disabled={!prefs.autoDeleteOldPrints}
                        onValueChange={(value) =>
                            onAutoDeletePrintsLimitChange(
                                value === null ? '' : String(value)
                            )
                        }
                        onValueCommitted={(value) =>
                            onAutoDeletePrintsLimitBlur(
                                value === null ? '' : String(value)
                            )
                        }
                    >
                        <NumberFieldGroup>
                            <NumberFieldDecrement />
                            <NumberFieldInput />
                            <NumberFieldIncrement />
                        </NumberFieldGroup>
                    </NumberField>
                </Field>
            </SettingsCard>
            <SettingsCard
                cardId="media.stickers"
                title={t(
                    'view.settings.advanced.advanced.save_instance_stickers_to_file.header'
                )}
            >
                <Field
                    label={t(
                        'view.settings.advanced.advanced.save_instance_stickers_to_file.description'
                    )}
                >
                    <Switch
                        checked={prefs.saveInstanceStickers}
                        onCheckedChange={onSaveInstanceStickersChange}
                    />
                </Field>
            </SettingsCard>
            <SettingsCard
                cardId="media.emoji"
                title={t(
                    'view.settings.advanced.advanced.save_instance_emoji_to_file.header'
                )}
                description={t(
                    'view.settings.advanced.advanced.save_instance_prints_to_file.header_tooltip'
                )}
            >
                <Field
                    label={t(
                        'view.settings.advanced.advanced.save_instance_emoji_to_file.description'
                    )}
                >
                    <Switch
                        checked={prefs.saveInstanceEmoji}
                        onCheckedChange={onSaveInstanceEmojiChange}
                    />
                </Field>
            </SettingsCard>
        </SettingsTabContent>
    );
}
