import { useEffect } from 'react';

import { requestGroupInstancesRefresh } from '@/services/runtime-event-bridge/auxiliaryEventHandlers';
import {
    setSidebarActiveTab,
    useSidebarTabStore
} from '@/state/sidebarTabStore';

export function useSidePanelActiveTab() {
    const activeTab = useSidebarTabStore((state) => state.activeTab);

    useEffect(() => {
        if (activeTab === 'groups') {
            void requestGroupInstancesRefresh('groups tab selected');
        }
    }, [activeTab]);

    return {
        activeTab,
        setActiveTab: setSidebarActiveTab
    };
}
