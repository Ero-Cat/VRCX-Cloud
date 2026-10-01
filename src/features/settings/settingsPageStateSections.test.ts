import { describe, expect, it, vi } from 'vitest';

import { createDefaultSettingsPrefs } from './settingsDefaultPrefs';
import {
    buildSettingsPageStateSections,
    type BuildSettingsPageStateSectionsInput
} from './settingsPageStateSections';

function createInput(
    overrides: Partial<BuildSettingsPageStateSectionsInput> = {}
): BuildSettingsPageStateSectionsInput {
    const callback = vi.fn();
    const asyncCallback = vi.fn(async () => undefined);
    const prefs = createDefaultSettingsPrefs();

    return {
        activeSettingsTab: 'system',
        commit: callback,
        handleCropInstancePrintsChange: callback,
        handleFeedPersistenceDisabledChange: callback,
        locale: 'en',
        normalizeRecentActionCooldownMinutes: () => 60,
        onlineVisitCount: null,
        openTableLimitsDialog: callback,
        openTablePageSizesDialog: callback,
        openUgcFolderSelector: callback,
        prefs,
        purgeAvatarFeedData: asyncCallback,
        purgeDialogOpen: false,
        purgeInProgress: false,
        purgePeriod: '180',
        refreshOnlineVisits: callback,
        refreshSqliteTableSizes: callback,
        resetTrustColors: callback,
        resetUgcFolder: callback,
        saveBoolPreference: callback,
        saveFontFamilyPreference: callback,
        saveInterfaceZoomLevel: callback,
        savePreferenceValue: callback,
        saveStringPreference: callback,
        saveTableLimitsDialog: callback,
        saveTrustColor: callback,
        saveWebhookActivityFilters: vi.fn(
            async () => prefs.webhookActivityFilters
        ),
        searchLimitError: '',
        selectCjkFontPack: callback,
        selectedFavoriteFriendGroupLabel: '',
        setAccessibleStatusIndicatorsPreference: callback,
        setActiveSettingsTab: callback,
        setAppLanguagePreference: callback,
        setDataTableStripedPreference: callback,
        setIntConfigPreference: callback,
        setNotificationLayoutPreference: callback,
        setPrefs: callback,
        setPurgeDialogOpen: callback,
        setProxyEnabledPreference: callback,
        setPurgePeriod: callback,
        setRecentActionCooldownEnabledPreference: callback,
        setRecentActionCooldownMinutesPreference: callback,
        setSaveInstanceEmojiPreference: callback,
        setSaveInstancePrintsPreference: callback,
        setSaveInstanceStickersPreference: callback,
        setShowNewDashboardButtonPreference: callback,
        setTableDensityPreference: callback,
        setTableLimitsDialogOpen: callback,
        setTableLimitsDraft: callback,
        setTablePageSizesDialogOpen: callback,
        setWebhookNotificationsDialogOpen: callback,
        setZoomInput: callback,
        setZoomLevelPreference: callback,
        sqliteTableSizes: {},
        toggleLocalFavoriteFriendsGroup: callback,
        webhookNotificationsDialogOpen: false,
        addFeedHiddenUser: callback,
        favoriteFriendGroupOptions: [],
        localFavoriteFriendGroupOptions: [],
        localFavoriteFriendsGroups: [],
        remoteFavoriteFriendGroupOptions: [],
        removeFeedHiddenUser: callback,
        tableLimitsDialogOpen: false,
        tableLimitsDraft: { maxTableSize: '10000', searchLimit: '100' },
        tableLimitsSaveDisabled: false,
        tableMaxSizeError: '',
        tablePageSizesDialogOpen: false,
        zoomInput: '100',
        zoomLevel: 1,
        ...overrides
    };
}

describe('settingsPageStateSections', () => {
    it('routes the key input values into their owning sections', () => {
        const prefs = createDefaultSettingsPrefs();
        const sections = buildSettingsPageStateSections(
            createInput({
                activeSettingsTab: 'interface',
                locale: 'ja',
                prefs,
                zoomInput: '125',
                zoomLevel: 1.25,
                tablePageSizesDialogOpen: true,
                tableLimitsDialogOpen: true,
                purgeDialogOpen: true
            })
        );

        expect(sections.shell).toMatchObject({
            activeSettingsTab: 'interface'
        });
        expect(sections.interface).toMatchObject({
            locale: 'ja',
            zoomInput: '125',
            zoomLevel: 1.25
        });
        expect(sections.social.feedHiddenUsers).toBe(prefs.feedHiddenUsers);
        expect(sections.advanced.sqliteTableSizes).toEqual({});
        expect(sections.dialogs).toMatchObject({
            tablePageSizesDialogOpen: true,
            tableLimitsDialogOpen: true,
            purgeDialogOpen: true
        });
        expect(sections.dialogs.webhookActivityFilters).toBe(
            prefs.webhookActivityFilters
        );
    });

    it('preserves interface callback routing', () => {
        const saveFontFamilyPreference = vi.fn();
        const sections = buildSettingsPageStateSections(
            createInput({
                saveFontFamilyPreference
            })
        );

        sections.interface.onFontFamilyChange('Inter');

        expect(saveFontFamilyPreference).toHaveBeenCalledWith('Inter');
    });

    it('routes user dialog appearance visibility through the interface section', () => {
        const saveBoolPreference = vi.fn();
        const sections = buildSettingsPageStateSections(
            createInput({
                activeSettingsTab: 'interface',
                saveBoolPreference
            })
        );

        sections.interface.onShowUserDialogProfileBackgroundChange(false);
        sections.interface.onShowUserDialogAvatarFrameChange(false);
        sections.interface.onShowUserDialogProfileEffectChange(false);
        sections.interface.onShowUserDialogNameplateEffectChange(false);

        for (const key of [
            'showUserDialogProfileBackground',
            'showUserDialogAvatarFrame',
            'showUserDialogProfileEffect',
            'showUserDialogNameplateEffect'
        ]) {
            expect(saveBoolPreference).toHaveBeenCalledWith(key, key, false);
        }
    });

    it('routes social bool preferences through the social section', () => {
        const saveBoolPreference = vi.fn();
        const sections = buildSettingsPageStateSections(
            createInput({
                activeSettingsTab: 'social',
                saveBoolPreference
            })
        );

        sections.social.onFriendLogNotificationDotChange(false);
        sections.social.onHideUnfriendsChange(true);
        sections.social.onProfileBioScanEnabledChange(true);

        expect(saveBoolPreference).toHaveBeenNthCalledWith(
            1,
            'friendLogNotificationDot',
            'friendLogNotificationDot',
            false
        );
        expect(saveBoolPreference).toHaveBeenCalledWith(
            'hideUnfriends',
            'hideUnfriends',
            true
        );
        expect(saveBoolPreference).toHaveBeenCalledWith(
            'profileBioScanEnabled',
            'profileBioScanEnabled',
            true
        );
    });
});
