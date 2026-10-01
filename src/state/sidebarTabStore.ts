import { create } from 'zustand';

import configRepository from '@/repositories/configRepository';
import {
    DEFAULT_SIDEBAR_TAB_LAYOUT,
    normalizeSidebarTabLayout,
    serializeSidebarTabLayout,
    upsertWorldRoomsTab,
    type SidebarTabLayout
} from '@/shared/utils/sidebarTabLayout';

type SidebarTabState = {
    tabLayout: SidebarTabLayout;
    tabLayoutHydrated: boolean;
    activeTab: string;
};

export const useSidebarTabStore = create<SidebarTabState>(() => ({
    tabLayout: DEFAULT_SIDEBAR_TAB_LAYOUT,
    tabLayoutHydrated: false,
    activeTab: 'friends'
}));

let tabLayoutHydration: Promise<void> | null = null;

export function hydrateSidebarTabLayout() {
    tabLayoutHydration ??= configRepository
        .getString('sidebarTabLayout', '[]')
        .then((storedLayout) => {
            if (!useSidebarTabStore.getState().tabLayoutHydrated) {
                useSidebarTabStore.setState({
                    tabLayout: normalizeSidebarTabLayout(storedLayout),
                    tabLayoutHydrated: true
                });
            }
        })
        .catch((error: unknown) => {
            tabLayoutHydration = null;
            throw error;
        });
    return tabLayoutHydration;
}

export function saveSidebarTabLayout(layout: SidebarTabLayout) {
    const tabLayout = normalizeSidebarTabLayout(layout);
    useSidebarTabStore.setState({ tabLayout, tabLayoutHydrated: true });
    return configRepository.setString(
        'sidebarTabLayout',
        serializeSidebarTabLayout(tabLayout)
    );
}

export function setSidebarActiveTab(activeTab: string) {
    useSidebarTabStore.setState({ activeTab });
}

export async function pinWorldRoomsTab(world: {
    worldId: string;
    name: string;
}) {
    await hydrateSidebarTabLayout();
    const { layout, tabId } = upsertWorldRoomsTab(
        useSidebarTabStore.getState().tabLayout,
        world
    );
    const saving = saveSidebarTabLayout(layout);
    setSidebarActiveTab(tabId);
    await saving;
}

export async function unpinWorldRoomsTab(worldId: string) {
    await hydrateSidebarTabLayout();
    const { tabLayout, activeTab } = useSidebarTabStore.getState();
    const removedIds = new Set(
        tabLayout
            .filter(
                (item) => item.type === 'worldRooms' && item.worldId === worldId
            )
            .map((item) => item.id)
    );
    const saving = saveSidebarTabLayout(
        tabLayout.filter((item) => !removedIds.has(item.id))
    );
    if (removedIds.has(activeTab)) {
        setSidebarActiveTab('friends');
    }
    await saving;
}
