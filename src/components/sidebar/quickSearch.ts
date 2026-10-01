import {
    commands,
    type QuickSearchEntityType,
    type QuickSearchQueryOutput,
    type QuickSearchResult as BackendQuickSearchResult
} from '@/platform/tauri/bindings';
import { convertFileUrlToImageUrl } from '@/services/entityMediaService';
import { isRecord } from '@/shared/utils/record';

export type { QuickSearchEntityType };

export type QuickSearchResult = Pick<
    BackendQuickSearchResult,
    'id' | 'type' | 'source' | 'name'
> & {
    subtitle?: string;
    imageUrl?: string;
    seedData?: Record<string, unknown> | null;
    memo?: string;
    note?: string;
    matchedField?: BackendQuickSearchResult['matchedField'];
    userColour?: string;
};

export type QuickSearchState = {
    status: QuickSearchQueryOutput['status'] | 'idle' | 'running' | 'error';
    detail: string;
    friends: QuickSearchResult[];
    ownAvatars: QuickSearchResult[];
    favoriteAvatars: QuickSearchResult[];
    ownWorlds: QuickSearchResult[];
    favoriteWorlds: QuickSearchResult[];
    ownGroups: QuickSearchResult[];
    joinedGroups: QuickSearchResult[];
};

function normalizeResult(result: BackendQuickSearchResult): QuickSearchResult {
    return {
        ...result,
        imageUrl: convertFileUrlToImageUrl(result.imageUrl, 64),
        seedData: isRecord(result.seedData) ? result.seedData : null
    };
}

function normalizeOutput(output: QuickSearchQueryOutput): QuickSearchState {
    return {
        ...output,
        friends: output.friends.map(normalizeResult),
        ownAvatars: output.ownAvatars.map(normalizeResult),
        favoriteAvatars: output.favoriteAvatars.map(normalizeResult),
        ownWorlds: output.ownWorlds.map(normalizeResult),
        favoriteWorlds: output.favoriteWorlds.map(normalizeResult),
        ownGroups: output.ownGroups.map(normalizeResult),
        joinedGroups: output.joinedGroups.map(normalizeResult)
    };
}

export function createEmptyQuickSearchState(
    status: QuickSearchState['status'] = 'idle',
    detail = ''
): QuickSearchState {
    return {
        status,
        detail,
        friends: [],
        ownAvatars: [],
        favoriteAvatars: [],
        ownWorlds: [],
        favoriteWorlds: [],
        ownGroups: [],
        joinedGroups: []
    };
}

export async function loadQuickSearchResults(
    query: string
): Promise<QuickSearchState> {
    return normalizeOutput(await commands.appQuickSearchQuery({ query }));
}
