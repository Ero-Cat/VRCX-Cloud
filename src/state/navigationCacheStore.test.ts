import { beforeEach, describe, expect, it, vi } from 'vitest';

const storage = vi.hoisted(() => {
    const backing = new Map<string, string>();
    return {
        backing,
        getItem: (key: string) => backing.get(key) ?? null,
        setItem: (key: string, value: string) => void backing.set(key, value),
        removeItem: (key: string) => void backing.delete(key),
        clear: () => backing.clear()
    };
});

vi.stubGlobal('window', {
    localStorage: storage,
    location: { hash: '' }
});

beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    storage.clear();
});

describe('navigation cache', () => {
    it('restores the route and independent open and closed folders', async () => {
        storage.backing.set(
            'vrcx-file:navigation-state.json',
            JSON.stringify({
                lastRoute: '/settings?tab=appearance',
                folders: { favorites: false, tools: true, invalid: 'true' },
                settingsCards: {
                    'system.application': false,
                    'advanced.troubleshooting': true,
                    invalid: 'true'
                },
                toolRows: { 'status-schedule': false, invalid: 1 }
            })
        );
        const { useNavigationCacheStore } =
            await import('./navigationCacheStore');
        await useNavigationCacheStore.getState().hydrate();
        const state = useNavigationCacheStore.getState();
        expect(state).toMatchObject({
            hydrated: true,
            lastRoute: '/settings?tab=appearance'
        });
        expect(state.folders).toEqual({ favorites: false, tools: true });
        expect(state.settingsCards).toEqual({
            'system.application': false,
            'advanced.troubleshooting': true
        });
        expect(state.toolRows).toEqual({ 'status-schedule': false });
    });

    it('keeps the stored value intact across hydrate-time writes', async () => {
        storage.backing.set(
            'vrcx-file:navigation-state.json',
            JSON.stringify({
                lastRoute: '/game-log',
                folders: { tools: true },
                settingsCards: { 'system.application': false }
            })
        );
        const { useNavigationCacheStore } =
            await import('./navigationCacheStore');
        await useNavigationCacheStore.getState().hydrate();
        expect(useNavigationCacheStore.getState().lastRoute).toBe('/game-log');
        expect(useNavigationCacheStore.getState().folders.tools).toBe(true);
        expect(
            useNavigationCacheStore.getState().settingsCards[
                'system.application'
            ]
        ).toBe(false);
    });

    it('continues with defaults on damaged cache and saves subsequent changes together', async () => {
        storage.backing.set('vrcx-file:navigation-state.json', '{');
        const { useNavigationCacheStore } =
            await import('./navigationCacheStore');
        await useNavigationCacheStore.getState().hydrate();
        expect(useNavigationCacheStore.getState().hydrated).toBe(true);
        useNavigationCacheStore.getState().setFolderOpen('favorites', false);
        useNavigationCacheStore.getState().setLastRoute('/friends-locations');
        useNavigationCacheStore
            .getState()
            .setToolRowOpen('status-schedule', false);
        await vi.waitFor(() =>
            expect(
                JSON.parse(
                    storage.backing.get('vrcx-file:navigation-state.json') ??
                        '{}'
                )
            ).toEqual({
                lastRoute: '/friends-locations',
                folders: { favorites: false },
                settingsCards: {},
                toolRows: { 'status-schedule': false }
            })
        );
    });

    it('serializes rapid card changes and restores the final states on restart', async () => {
        storage.backing.set(
            'vrcx-file:navigation-state.json',
            JSON.stringify({ lastRoute: '/settings', folders: { tools: true } })
        );
        const { useNavigationCacheStore } =
            await import('./navigationCacheStore');
        await useNavigationCacheStore.getState().hydrate();
        expect(useNavigationCacheStore.getState().settingsCards).toEqual({});
        const { setSettingsCardOpen } = useNavigationCacheStore.getState();
        setSettingsCardOpen('system.application', false);
        setSettingsCardOpen('advanced.troubleshooting', true);
        setSettingsCardOpen('system.application', true);
        setSettingsCardOpen('system.application', false);
        await vi.waitFor(() => {
            const saved: string =
                storage.backing.get('vrcx-file:navigation-state.json') ?? '';
            expect(JSON.parse(saved)).toEqual({
                lastRoute: '/settings',
                folders: { tools: true },
                settingsCards: {
                    'system.application': false,
                    'advanced.troubleshooting': true
                },
                toolRows: {}
            });
        });
        storage.backing.set(
            'vrcx-file:navigation-state.json',
            JSON.stringify({
                lastRoute: '/settings',
                folders: { tools: true },
                settingsCards: {
                    'system.application': false,
                    'advanced.troubleshooting': true
                },
                toolRows: {}
            })
        );
        vi.resetModules();
        const restarted = (await import('./navigationCacheStore'))
            .useNavigationCacheStore;
        await restarted.getState().hydrate();
        expect(restarted.getState().settingsCards).toEqual({
            'system.application': false,
            'advanced.troubleshooting': true
        });
    });
});
