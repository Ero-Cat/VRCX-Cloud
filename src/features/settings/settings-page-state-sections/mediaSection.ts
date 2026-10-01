import { openUGCPhotosFolder } from '@/services/shellIntegrationService';
import { normalizeAutoDeletePrintsLimit } from '@/state/preferencesStore';

import type { SettingsSectionInput } from '../settingsPageStateSectionTypes';

type MediaSectionInput = SettingsSectionInput<
    | 'prefs'
    | 'commit'
    | 'setScreenshotHelperPreference'
    | 'setScreenshotHelperModifyFilenamePreference'
    | 'setScreenshotHelperCopyToClipboardPreference'
    | 'deleteAllScreenshotMetadata'
    | 'openUgcFolderSelector'
    | 'resetUgcFolder'
    | 'setSaveInstancePrintsPreference'
    | 'handleCropInstancePrintsChange'
    | 'setSaveInstanceStickersPreference'
    | 'setSaveInstanceEmojiPreference'
    | 'setPrefs'
    | 'savePreferenceValue'
    | 'saveBoolPreference'
    | 'setIntConfigPreference'
>;

export function buildMediaSection({
    prefs,
    commit,
    setScreenshotHelperPreference,
    setScreenshotHelperModifyFilenamePreference,
    setScreenshotHelperCopyToClipboardPreference,
    deleteAllScreenshotMetadata,
    openUgcFolderSelector,
    resetUgcFolder,
    setSaveInstancePrintsPreference,
    handleCropInstancePrintsChange,
    setSaveInstanceStickersPreference,
    setSaveInstanceEmojiPreference,
    setPrefs,
    savePreferenceValue,
    saveBoolPreference,
    setIntConfigPreference
}: MediaSectionInput) {
    return {
        commit,
        setScreenshotHelperPreference,
        setScreenshotHelperModifyFilenamePreference,
        setScreenshotHelperCopyToClipboardPreference,
        deleteAllScreenshotMetadata,
        openUgcFolderSelector,
        resetUgcFolder,
        setSaveInstancePrintsPreference,
        handleCropInstancePrintsChange,
        setSaveInstanceStickersPreference,
        setSaveInstanceEmojiPreference,
        setPrefs,
        onScreenshotHelperChange: (checked: boolean) => {
            savePreferenceValue('screenshotHelper', checked, () =>
                setScreenshotHelperPreference(checked)
            );
        },
        onScreenshotHelperModifyFilenameChange: (checked: boolean) => {
            savePreferenceValue('screenshotHelperModifyFilename', checked, () =>
                setScreenshotHelperModifyFilenamePreference(checked)
            );
        },
        onScreenshotHelperCopyToClipboardChange: (checked: boolean) => {
            savePreferenceValue(
                'screenshotHelperCopyToClipboard',
                checked,
                () => setScreenshotHelperCopyToClipboardPreference(checked)
            );
        },
        onDeleteAllScreenshotMetadata: () => {
            deleteAllScreenshotMetadata();
        },
        onOpenUgcPhotosFolder: () => {
            commit(() => openUGCPhotosFolder(prefs.userGeneratedContentPath));
        },
        onOpenUgcFolderSelector: () => {
            openUgcFolderSelector();
        },
        onResetUgcFolder: () => {
            resetUgcFolder();
        },
        onSaveInstancePrintsChange: (checked: boolean) => {
            savePreferenceValue('saveInstancePrints', checked, () =>
                setSaveInstancePrintsPreference(checked)
            );
        },
        onCropInstancePrintsChange: (checked: boolean) => {
            handleCropInstancePrintsChange(checked);
        },
        onAutoDeleteOldPrintsChange: (checked: boolean) => {
            saveBoolPreference(
                'autoDeleteOldPrints',
                'autoDeleteOldPrints',
                checked
            );
        },
        onAutoDeletePrintsLimitChange: (value: string) => {
            setPrefs((current) => ({
                ...current,
                autoDeletePrintsLimit: value
            }));
        },
        onAutoDeletePrintsLimitBlur: (value: string) => {
            const nextValue = normalizeAutoDeletePrintsLimit(value);
            savePreferenceValue('autoDeletePrintsLimit', nextValue, () =>
                setIntConfigPreference('autoDeletePrintsLimit', nextValue, {
                    min: 30,
                    max: 60,
                    fallback: 60
                })
            );
        },
        onSaveInstanceStickersChange: (checked: boolean) => {
            savePreferenceValue('saveInstanceStickers', checked, () =>
                setSaveInstanceStickersPreference(checked)
            );
        },
        onSaveInstanceEmojiChange: (checked: boolean) => {
            savePreferenceValue('saveInstanceEmoji', checked, () =>
                setSaveInstanceEmojiPreference(checked)
            );
        }
    };
}
