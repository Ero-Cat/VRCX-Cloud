import type { HostCapabilities } from '@/platform/native/bindings';
import {
    getHostCapabilityUnavailableReason,
    isHostCapabilityAvailable,
    isHostCapabilitySupported
} from '@/services/hostCapabilityService';
import i18n from '@/services/i18nService';
import { recordToolOpen } from '@/services/telemetry/telemetryToolUsage';
import { toast } from '@/services/toastService';
import { recordRecentToolOpen } from '@/services/toolRecentService';
import {
    toolDefinitionMap,
    type ToolDialogKey,
    type ToolDefinition,
    type ToolRouteName
} from '@/shared/constants/tools';
import { useRuntimeStore } from '@/state/runtimeStore';

type Navigate = (to: string) => void;
type Translate = (key: string) => string;
type TriggerToolOptions = {
    navigate: Navigate;
    t: Translate;
};
type ToolDialogHostKey =
    | 'appLauncherOpen'
    | 'presenceScheduleOpen'
    | 'presenceRoomRulesOpen'
    | 'presenceInviteRequestsOpen'
    | 'groupCalendarOpen'
    | 'exportDiscordNamesOpen'
    | 'noteExportOpen'
    | 'exportFriendsListOpen'
    | 'exportAvatarsListOpen'
    | 'editInviteMessagesOpen'
    | 'llmEndpointsOpen'
    | 'profileBackupOpen';

const toolRouteMap = {
    gallery: '/tools/gallery',
    inventory: '/tools/inventory',
    'group-moderation': '/tools/group-moderation',
    'my-groups': '/tools/my-groups'
} satisfies Record<ToolRouteName, string>;

const toolDialogHostMap: Record<ToolDialogKey, ToolDialogHostKey> = {
    'presence-schedule': 'presenceScheduleOpen',
    'presence-room-rules': 'presenceRoomRulesOpen',
    'presence-invite-requests': 'presenceInviteRequestsOpen',
    'group-calendar': 'groupCalendarOpen',
    'export-discord-names': 'exportDiscordNamesOpen',
    'note-export': 'noteExportOpen',
    'export-friends-list': 'exportFriendsListOpen',
    'export-avatars-list': 'exportAvatarsListOpen',
    'edit-invite-messages': 'editInviteMessagesOpen',
    'llm-endpoints': 'llmEndpointsOpen',
    'profile-backup': 'profileBackupOpen'
};

const legacyToolAliases: Record<string, string> = {
    'auto-change-status': 'presence-room-rules'
};

export function isToolCapabilityAvailable(
    tool?: ToolDefinition | null,
    hostCapabilities?: HostCapabilities
): boolean {
    const capabilities = [
        ...(tool?.requiredCapabilities ?? []),
        ...(tool?.requiredCapability ? [tool.requiredCapability] : [])
    ];
    if (capabilities.length === 0) {
        return true;
    }
    if (hostCapabilities) {
        return capabilities.every((capability) => {
            const status = hostCapabilities[capability];
            return tool?.requiredCapabilityMode === 'supported'
                ? status.supported && status.enabled
                : status.available;
        });
    }
    if (tool?.requiredCapabilityMode === 'supported') {
        return capabilities.every(isHostCapabilitySupported);
    }
    return capabilities.every(isHostCapabilityAvailable);
}

function getToolCapabilityUnavailableReason(
    tool?: ToolDefinition | null
): string {
    const capabilities = [
        ...(tool?.requiredCapabilities ?? []),
        ...(tool?.requiredCapability ? [tool.requiredCapability] : [])
    ];
    if (capabilities.length === 0) {
        return '';
    }
    const capability = capabilities[0];
    return getHostCapabilityUnavailableReason(capability);
}

export async function triggerToolByKey(
    toolKey: string,
    { navigate, t: _t }: TriggerToolOptions
): Promise<void> {
    const resolvedToolKey = legacyToolAliases[toolKey] ?? toolKey;
    const tool = toolDefinitionMap.get(resolvedToolKey);
    const action = tool?.action;
    if (!action) {
        toast.add({
            type: 'error',
            title: i18n.t(
                'service.tool_action_service.dynamic.unknown_tool_action_value',
                { value: toolKey }
            )
        });
        return;
    }

    if (!isToolCapabilityAvailable(tool)) {
        toast.add({
            type: 'error',
            title: getToolCapabilityUnavailableReason(tool)
        });
        return;
    }

    recordToolOpen(resolvedToolKey);
    void recordRecentToolOpen(resolvedToolKey).catch(() => {});

    if (action.type === 'route') {
        navigate(toolRouteMap[action.routeName] ?? '/tools');
        return;
    }

    if (action.type === 'dialog') {
        useRuntimeStore
            .getState()
            .setSystemHostOpen(toolDialogHostMap[action.dialogKey], true);
        return;
    }

    toast.add({
        type: 'error',
        title: i18n.t(
            'service.tool_action_service.dynamic.unsupported_tool_action_value',
            { value: toolKey }
        )
    });
}
