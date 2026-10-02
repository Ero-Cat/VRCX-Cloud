import {
    commands,
    type FeedBioHistoryOutput,
    type StatusStatsViewOutput
} from '@/platform/native/bindings';

function utcOffsetMinutes() {
    return -new Date().getTimezoneOffset();
}

export const socialAnalyticsService = Object.freeze({
    async loadBioHistory(input: {
        ownerUserId: string;
        targetUserId: string;
        limit?: number;
    }): Promise<FeedBioHistoryOutput> {
        if (!input.ownerUserId || !input.targetUserId) {
            return { rows: [] };
        }
        return commands.appFeedBioHistoryQuery({
            userId: input.ownerUserId,
            targetUserId: input.targetUserId,
            limit: input.limit ?? 50
        });
    },

    async loadStatusStats(input: {
        ownerUserId: string;
        targetUserId: string;
        rangeDays?: number;
        isSelf?: boolean;
    }): Promise<StatusStatsViewOutput> {
        if (!input.ownerUserId || !input.targetUserId) {
            return {
                rangeDays: 0,
                totals: [],
                daily: [],
                trackedMinutes: 0,
                hasAnyData: false,
                builtAt: ''
            };
        }
        return commands.appStatusStatsView({
            ownerUserId: input.ownerUserId,
            targetUserId: input.targetUserId,
            rangeDays: input.rangeDays ?? 90,
            utcOffsetMinutes: utcOffsetMinutes(),
            isSelf: input.isSelf ?? false
        });
    }
});
