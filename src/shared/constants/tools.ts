type ToolCategoryKey =
    | 'image'
    | 'shortcuts'
    | 'automation'
    | 'group'
    | 'vrchat'
    | 'data'
    | 'debug'
    | 'other';

export type ToolDialogKey =
    | 'presence-schedule'
    | 'presence-room-rules'
    | 'presence-invite-requests'
    | 'group-calendar'
    | 'export-discord-names'
    | 'note-export'
    | 'export-friends-list'
    | 'export-avatars-list'
    | 'edit-invite-messages'
    | 'llm-endpoints'
    | 'profile-backup';

type ToolHostCapabilityKey =
    | 'gameLaunch'
    | 'gameProcessMonitor'
    | 'registryPrefs'
    | 'screenshotCache'
    | 'vrchatPathDiscovery';

export type ToolRouteName =
    | 'gallery'
    | 'inventory'
    | 'group-moderation'
    | 'my-groups';

type ToolAction =
    | {
          type: 'route';
          routeName: ToolRouteName;
      }
    | {
          type: 'dialog';
          dialogKey: ToolDialogKey;
      };

interface ToolCategory {
    key: ToolCategoryKey;
    labelKey: string;
}

interface ToolDefinition {
    key: string;
    category: ToolCategoryKey;
    iconKey: string;
    navIcon: string;
    titleKey: string;
    descriptionKey: string;
    navEligible: boolean;
    requiredCapability?: ToolHostCapabilityKey;
    requiredCapabilities?: ToolHostCapabilityKey[];
    requiredCapabilityMode?: 'supported';
    action: ToolAction;
}

interface ToolNavDefinition {
    key: string;
    icon: string;
    tooltip: string;
    labelKey: string;
    routeName: string | null;
    action: { type: 'tool'; toolKey: string } | null;
    defaultHidden: boolean;
}

const toolCategories: ToolCategory[] = [
    { key: 'image', labelKey: 'view.tools.pictures.header' },
    { key: 'shortcuts', labelKey: 'view.tools.shortcuts.header' },
    {
        key: 'automation',
        labelKey: 'view.tools.category.automation'
    },
    { key: 'group', labelKey: 'view.tools.group.header' },
    { key: 'vrchat', labelKey: 'view.tools.category.vrchat' },
    { key: 'data', labelKey: 'view.tools.category.data' },
    { key: 'debug', labelKey: 'view.tools.category.debug' },
    { key: 'other', labelKey: 'view.tools.other.header' }
];

const toolDefinitions: ToolDefinition[] = [
    {
        key: 'gallery',
        category: 'image',
        iconKey: 'image',
        navIcon: 'lucide:Images',
        titleKey: 'view.tools.pictures.gallery',
        descriptionKey: 'view.tools.pictures.gallery_description',
        navEligible: true,
        action: { type: 'route', routeName: 'gallery' }
    },
    {
        key: 'inventory',
        category: 'image',
        iconKey: 'package',
        navIcon: 'lucide:Package',
        titleKey: 'view.tools.pictures.inventory',
        descriptionKey: 'view.tools.pictures.inventory_description',
        navEligible: true,
        action: { type: 'route', routeName: 'inventory' }
    },
    {
        key: 'profile-backup',
        category: 'data',
        iconKey: 'database-backup',
        navIcon: 'lucide:DatabaseBackup',
        titleKey: 'profile_backup.header',
        descriptionKey: 'profile_backup.tools_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'profile-backup' }
    },
    {
        key: 'llm-endpoints',
        category: 'other',
        iconKey: 'plug',
        navIcon: 'lucide:Plug',
        titleKey: 'view.tools.system_tools.llm_endpoints',
        descriptionKey: 'view.tools.system_tools.llm_endpoints_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'llm-endpoints' }
    },
    {
        key: 'presence-schedule',
        category: 'automation',
        iconKey: 'calendar',
        navIcon: 'lucide:CalendarDays',
        titleKey: 'view.tools.social_automation.status_schedule',
        descriptionKey:
            'view.tools.social_automation.status_schedule_description',
        navEligible: true,
        action: {
            type: 'dialog',
            dialogKey: 'presence-schedule'
        }
    },
    {
        key: 'presence-room-rules',
        category: 'automation',
        iconKey: 'users',
        navIcon: 'lucide:UsersRound',
        titleKey: 'view.tools.social_automation.room_status_rules',
        descriptionKey:
            'view.tools.social_automation.room_status_rules_description',
        navEligible: true,
        action: {
            type: 'dialog',
            dialogKey: 'presence-room-rules'
        }
    },
    {
        key: 'presence-invite-requests',
        category: 'automation',
        iconKey: 'message',
        navIcon: 'lucide:MessageSquareText',
        titleKey: 'view.tools.social_automation.invite_request_auto_reply',
        descriptionKey:
            'view.tools.social_automation.invite_request_auto_reply_description',
        navEligible: true,
        action: {
            type: 'dialog',
            dialogKey: 'presence-invite-requests'
        }
    },
    {
        key: 'group-calendar',
        category: 'group',
        iconKey: 'calendar',
        navIcon: 'lucide:CalendarDays',
        titleKey: 'view.tools.group.calendar',
        descriptionKey: 'view.tools.group.calendar_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'group-calendar' }
    },
    {
        key: 'my-groups',
        category: 'group',
        iconKey: 'users-round',
        navIcon: 'lucide:UsersRound',
        titleKey: 'view.tools.group.my_groups',
        descriptionKey: 'view.tools.group.my_groups_description',
        navEligible: true,
        action: { type: 'route', routeName: 'my-groups' }
    },
    {
        key: 'group-moderation',
        category: 'group',
        iconKey: 'shield-user',
        navIcon: 'lucide:ShieldUser',
        titleKey: 'view.tools.group.moderation',
        descriptionKey: 'view.tools.group.moderation_description',
        navEligible: true,
        action: { type: 'route', routeName: 'group-moderation' }
    },
    {
        key: 'discord-names',
        category: 'data',
        iconKey: 'users',
        navIcon: 'lucide:Users',
        titleKey: 'view.tools.export.discord_names',
        descriptionKey: 'view.tools.user.discord_names_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'export-discord-names' }
    },
    {
        key: 'export-notes',
        category: 'data',
        iconKey: 'file-text',
        navIcon: 'lucide:FileText',
        titleKey: 'view.tools.export.export_notes',
        descriptionKey: 'view.tools.export.export_notes_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'note-export' }
    },
    {
        key: 'export-friend-list',
        category: 'data',
        iconKey: 'users',
        navIcon: 'lucide:Contact',
        titleKey: 'view.tools.export.export_friend_list',
        descriptionKey: 'view.tools.user.export_friend_list_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'export-friends-list' }
    },
    {
        key: 'export-own-avatars',
        category: 'data',
        iconKey: 'download',
        navIcon: 'lucide:Download',
        titleKey: 'view.tools.export.export_own_avatars',
        descriptionKey: 'view.tools.user.export_own_avatars_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'export-avatars-list' }
    },
    {
        key: 'edit-invite-message',
        category: 'other',
        iconKey: 'pencil',
        navIcon: 'lucide:MessageSquareText',
        titleKey: 'view.tools.other.edit_invite_message',
        descriptionKey: 'view.tools.other.edit_invite_message_description',
        navEligible: true,
        action: { type: 'dialog', dialogKey: 'edit-invite-messages' }
    }
];

const toolDefinitionMap = new Map<string, ToolDefinition>(
    toolDefinitions.map((tool) => [tool.key, tool])
);

const quickAccessConfigKey = 'VRCX_toolsQuickAccessList';
const recentToolsConfigKey = 'VRCX_toolsRecentList';
const TOOLS_QUICK_ACCESS_UPDATED_EVENT = 'vrcx:tools-quick-access-updated';
const TOOLS_STATUS_UPDATED_EVENT = 'vrcx:tools-status-updated';
const RECENT_TOOLS_LIMIT = 3;
const knownToolKeys = new Set(toolDefinitions.map((tool) => tool.key));
const legacyToolKeyAliases: Record<string, string> = {
    'auto-change-status': 'presence-room-rules'
};

function normalizePinnedToolKey(toolKey: unknown): string {
    const normalizedToolKey = String(toolKey ?? '');
    return legacyToolKeyAliases[normalizedToolKey] ?? normalizedToolKey;
}

function normalizeQuickAccessToolKeys(value: unknown): string[] {
    if (!Array.isArray(value)) {
        return [];
    }

    const seen = new Set<string>();
    const nextKeys: string[] = [];
    for (const rawKey of value) {
        const toolKey = normalizePinnedToolKey(String(rawKey || ''));
        if (!knownToolKeys.has(toolKey) || seen.has(toolKey)) {
            continue;
        }
        seen.add(toolKey);
        nextKeys.push(toolKey);
    }
    return nextKeys;
}

function parseQuickAccessToolKeys(value: unknown): string[] {
    try {
        return normalizeQuickAccessToolKeys(JSON.parse(String(value || '[]')));
    } catch {
        return [];
    }
}

function normalizeRecentToolKeys(value: unknown): string[] {
    return normalizeQuickAccessToolKeys(value).slice(0, RECENT_TOOLS_LIMIT);
}

function parseRecentToolKeys(value: unknown): string[] {
    try {
        return normalizeRecentToolKeys(JSON.parse(String(value || '[]')));
    } catch {
        return [];
    }
}

function getEquivalentToolNavKeys(toolKey: unknown): string[] {
    const normalizedToolKey = normalizePinnedToolKey(toolKey);
    const equivalentToolKeys = new Set([normalizedToolKey]);

    for (const [legacyToolKey, targetToolKey] of Object.entries(
        legacyToolKeyAliases
    )) {
        if (targetToolKey === normalizedToolKey) {
            equivalentToolKeys.add(legacyToolKey);
        }
    }

    return Array.from(equivalentToolKeys).map((key) => `tool-${key}`);
}

function publishToolsQuickAccessUpdated(): void {
    if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent(TOOLS_QUICK_ACCESS_UPDATED_EVENT));
    }
}

function publishToolsStatusUpdated(): void {
    if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent(TOOLS_STATUS_UPDATED_EVENT));
    }
}

const generatedToolNavDefinitions: ToolNavDefinition[] = toolDefinitions
    .filter((tool) => tool.navEligible)
    .map((tool) => ({
        key: `tool-${tool.key}`,
        icon: tool.navIcon,
        tooltip: tool.titleKey,
        labelKey: tool.titleKey,
        routeName: tool.action.type === 'route' ? tool.action.routeName : null,
        action: {
            type: 'tool',
            toolKey: tool.key
        },
        defaultHidden: true
    }));

const legacyToolNavDefinitions: ToolNavDefinition[] = [
    {
        key: 'tool-auto-change-status',
        icon: 'lucide:Bot',
        tooltip: 'view.tools.social_automation.room_status_rules',
        labelKey: 'view.tools.social_automation.room_status_rules',
        routeName: null,
        action: { type: 'tool', toolKey: 'auto-change-status' },
        defaultHidden: true
    }
];

const toolNavDefinitions: ToolNavDefinition[] = [
    ...generatedToolNavDefinitions,
    ...legacyToolNavDefinitions
];

const isToolNavKey = (key: unknown): key is string =>
    typeof key === 'string' && key.startsWith('tool-');

function getToolsByCategory(categoryKey: ToolCategoryKey): ToolDefinition[] {
    return toolDefinitions.filter((tool) => tool.category === categoryKey);
}

export {
    TOOLS_STATUS_UPDATED_EVENT,
    getEquivalentToolNavKeys,
    isToolNavKey,
    knownToolKeys,
    normalizePinnedToolKey,
    normalizeQuickAccessToolKeys,
    normalizeRecentToolKeys,
    parseQuickAccessToolKeys,
    parseRecentToolKeys,
    publishToolsQuickAccessUpdated,
    publishToolsStatusUpdated,
    quickAccessConfigKey,
    recentToolsConfigKey,
    toolCategories,
    toolDefinitions,
    toolDefinitionMap,
    toolNavDefinitions,
    getToolsByCategory
};
export type { ToolDefinition };
