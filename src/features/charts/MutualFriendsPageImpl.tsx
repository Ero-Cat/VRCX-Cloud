import { LinkIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { PageScaffold } from '@/components/layout/PageScaffold';
import { useRoomMutualScanStore } from '@/state/roomMutualScanStore';
import { Button } from '@/ui/shadcn/button';

import { ManualRelationsDialog } from './components/mutual-friends/ManualRelationsDialog';
import { MutualFriendsAutoFetch } from './components/mutual-friends/MutualFriendsAutoFetch';
import { MutualFriendsHud } from './components/mutual-friends/MutualFriendsHud';
import { MutualFriendsLegend } from './components/mutual-friends/MutualFriendsLegend';
import { MutualFriendsNodeCard } from './components/mutual-friends/MutualFriendsNodeCard';
import { MutualFriendsSettingsSheet } from './components/mutual-friends/MutualFriendsSettingsSheet';
import {
    MutualFriendsLayoutBadge,
    MutualFriendsStageOverlay
} from './components/mutual-friends/MutualFriendsStageOverlay';
import { useMutualFriendsPageState } from './mutual-friends/useMutualFriendsPageState';

export function MutualFriendsPage() {
    const { t } = useTranslation();
    const [manualRelationsOpen, setManualRelationsOpen] = useState(false);
    const {
        actions,
        exclusions,
        extras,
        fetch,
        graph,
        layout,
        selection,
        view
    } = useMutualFriendsPageState();
    const roomScanRunning = useRoomMutualScanStore((state) => state.running);

    const hasActiveFilters = Boolean(
        view.filters.searchQuery ||
        view.filters.minDegree > 0 ||
        view.filters.focusedCommunity !== null
    );
    const selectedNode = selection.node;
    const selectedCommunity =
        selection.communityIndex === null
            ? null
            : (graph.communities.find(
                  (community) => community.index === selection.communityIndex
              ) ?? null);

    return (
        <PageScaffold id="chart" className="p-0">
            <MutualFriendsAutoFetch
                currentUserId={graph.currentUserId}
                onFetch={actions.fetchGraph}
            />
            <div className="relative min-h-0 flex-1 overflow-hidden">
                <div
                    ref={graph.setGraphElementRef}
                    className="absolute inset-0"
                />

                <MutualFriendsHud
                    baseNodeCount={graph.baseNodeCount}
                    canFetch={Boolean(graph.currentUserId) && !roomScanRunning}
                    fetchProgress={fetch.fetchProgress}
                    isReloading={
                        graph.status === 'running' && graph.baseNodeCount > 0
                    }
                    onCancelFetch={actions.cancelFetch}
                    onFetchGraph={actions.fetchGraph}
                    onRefreshPage={actions.refreshPage}
                    onSearchQueryChange={actions.setSearchQuery}
                    searchQuery={view.filters.searchQuery}
                    actionsSlot={
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon-sm"
                            aria-label={t(
                                'view.charts.mutual_friend.manual_relations.button_tooltip'
                            )}
                            onClick={() => setManualRelationsOpen(true)}
                        >
                            <LinkIcon />
                        </Button>
                    }
                    settingsSlot={
                        <MutualFriendsSettingsSheet
                            edgeCount={graph.edgeCount}
                            excludePickerOptions={
                                exclusions.excludePickerOptions
                            }
                            excludedFriendIds={exclusions.excludedFriendIds}
                            externalUsers={extras?.externalUsers ?? []}
                            layoutSettings={layout.layoutSettings}
                            nodeCount={graph.nodeCount}
                            onAddExternalUser={actions.addExternalUser}
                            onExcludedFriendIdsChange={
                                exclusions.setExcludedFriendIds
                            }
                            onRemoveExternalUser={actions.removeExternalUser}
                            onResetLayoutAndHidden={
                                actions.resetLayoutAndHidden
                            }
                            setLayoutSetting={layout.setLayoutSetting}
                        />
                    }
                />

                {graph.isLayoutRunning && graph.nodeCount ? (
                    <MutualFriendsLayoutBadge />
                ) : null}

                {graph.baseNodeCount > 0 && graph.nodeCount > 0 ? (
                    <MutualFriendsLegend
                        communities={graph.communities}
                        coverage={graph.coverage}
                        crossCommunityOnly={view.crossCommunityOnly}
                        focusedCommunity={view.filters.focusedCommunity}
                        isolatedCounts={graph.isolatedCounts}
                        minDegree={view.filters.minDegree}
                        onMinDegreeChange={actions.setMinDegree}
                        onToggleCrossCommunityOnly={
                            actions.toggleCrossCommunityOnly
                        }
                        onToggleFocusedCommunity={
                            actions.toggleFocusedCommunity
                        }
                        unknownCount={graph.unknownCount}
                    />
                ) : null}

                {selectedNode ? (
                    <MutualFriendsNodeCard
                        community={selectedCommunity}
                        isRefreshing={selection.isRefreshing}
                        linkableNodeOptions={extras?.linkableNodeOptions ?? []}
                        node={selectedNode}
                        onAddManualLink={(targetId) =>
                            actions.addManualLink(selectedNode.id, targetId)
                        }
                        onClose={actions.clearSelection}
                        onFocusCommunity={() => {
                            if (selection.communityIndex !== null) {
                                actions.toggleFocusedCommunity(
                                    selection.communityIndex
                                );
                            }
                        }}
                        onHide={() =>
                            actions.toggleExcludedFriendId(selectedNode.id)
                        }
                        onOpenProfile={() => actions.openNode(selectedNode.id)}
                        onRefresh={actions.refreshSelectedNode}
                        user={selection.user}
                    />
                ) : null}

                <ManualRelationsDialog
                    open={manualRelationsOpen}
                    onOpenChange={setManualRelationsOpen}
                    manualLinks={extras?.manualLinks ?? []}
                    onChanged={actions.refreshPage}
                />

                <MutualFriendsStageOverlay
                    baseNodeCount={graph.baseNodeCount}
                    detail={graph.detail}
                    hasActiveFilters={hasActiveFilters}
                    nodeCount={graph.nodeCount}
                    onLoadConnections={actions.fetchGraph}
                    onClearFilters={actions.clearFilters}
                    status={graph.status}
                />
            </div>
        </PageScaffold>
    );
}
