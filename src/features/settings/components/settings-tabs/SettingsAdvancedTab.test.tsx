// @vitest-environment jsdom

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { Tabs } from '@/ui/shadcn/tabs';
import { TooltipProvider } from '@/ui/shadcn/tooltip';

import { SettingsAdvancedTabContent as SettingsAdvancedTab } from './SettingsAdvancedTab';
import type { SettingsAdvancedModel } from './settingsAdvancedTypes';

const labels: Record<string, string> = {
    'view.settings.advanced.advanced_ui.behavior.deep_link_registration':
        'Open VRCX-Cloud links',
    'view.settings.advanced.advanced_ui.behavior.deep_link_repair': 'Fix'
};

const commandMocks = vi.hoisted(() => ({
    appBrowseHistoryRetentionDaysGet: vi.fn(),
    appDeepLinkRegistrationStatus: vi.fn(),
    appDeepLinkRegistrationRepair: vi.fn()
}));

vi.mock('@/platform/tauri/bindings', () => ({
    commands: commandMocks
}));

vi.mock('react-i18next', async (importOriginal) => ({
    ...(await importOriginal<typeof import('react-i18next')>()),
    useTranslation: () => ({
        t: (key: string) => labels[key] ?? key
    })
}));

vi.mock('./AdvancedTroubleshootingGroup', () => ({
    AdvancedTroubleshootingGroup: () => <div>troubleshooting</div>
}));

function createModel(
    overrides: Partial<SettingsAdvancedModel> = {}
): SettingsAdvancedModel {
    return {
        avatarAutoCleanupOptions: ['Off'],
        onAvatarAutoCleanupChange: vi.fn(),
        onFeedPersistenceDisabledChange: vi.fn(),
        onLogResourceLoadChange: vi.fn(),
        onOpenPurgeDialog: vi.fn(),
        onRefreshOnlineVisits: vi.fn(),
        onRefreshSqliteTableSizes: vi.fn(),
        onUdonExceptionLoggingChange: vi.fn(),
        onlineVisitCount: null,
        prefs: {
            avatarAutoCleanup: 'Off',
            feedPersistenceDisabled: false,
            logResourceLoad: false,
            udonExceptionLogging: false
        },
        sqliteTableSizeRows: [],
        sqliteTableSizes: {},
        ...overrides
    };
}

function renderTab(model: SettingsAdvancedModel) {
    return render(
        <TooltipProvider>
            <Tabs value="advanced">
                <SettingsAdvancedTab advanced={model} />
            </Tabs>
        </TooltipProvider>
    );
}

describe('SettingsAdvancedTab deep link registration', () => {
    afterEach(cleanup);

    beforeEach(() => {
        commandMocks.appBrowseHistoryRetentionDaysGet
            .mockReset()
            .mockResolvedValue(30);
        commandMocks.appDeepLinkRegistrationStatus
            .mockReset()
            .mockResolvedValue(null);
        commandMocks.appDeepLinkRegistrationRepair
            .mockReset()
            .mockResolvedValue(true);
        vi.stubGlobal(
            'ResizeObserver',
            class {
                observe() {}
                unobserve() {}
                disconnect() {}
            }
        );
    });

    it('shows the cross-platform Fix action for registration errors', async () => {
        commandMocks.appDeepLinkRegistrationStatus.mockRejectedValueOnce(
            new Error('registry value is malformed')
        );

        renderTab(createModel());

        expect(
            await screen.findByRole('button', {
                name: 'Fix'
            })
        ).not.toBeNull();
        expect(screen.getByText('Open VRCX-Cloud links')).not.toBeNull();
    });

    it('keeps the repair action hidden on unsupported platforms', async () => {
        renderTab(createModel());

        await vi.waitFor(() => {
            expect(
                commandMocks.appDeepLinkRegistrationStatus
            ).toHaveBeenCalledOnce();
        });
        expect(
            screen.queryByRole('button', {
                name: 'Fix'
            })
        ).toBeNull();
    });
});
