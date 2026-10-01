import {
    isSameBoopEmoji,
    parseBoopEmojiChoice,
    type BoopEmojiChoice
} from '@/domain/entities/boopEmoji';
import configRepository from '@/repositories/configRepository';

const RECENT_BOOP_EMOJIS_CONFIG_KEY = 'VRCX_boopRecentEmojis';
const RECENT_BOOP_EMOJIS_LIMIT = 10;

let writeQueue = Promise.resolve();

function normalizeRecentBoopEmojis(value: unknown): BoopEmojiChoice[] {
    if (!Array.isArray(value)) {
        return [];
    }
    const next: BoopEmojiChoice[] = [];
    for (const entry of value) {
        const choice = parseBoopEmojiChoice(entry);
        if (choice && !next.some((known) => isSameBoopEmoji(known, choice))) {
            next.push(choice);
        }
    }
    return next.slice(0, RECENT_BOOP_EMOJIS_LIMIT);
}

function parseRecentBoopEmojis(value: unknown): BoopEmojiChoice[] {
    try {
        return normalizeRecentBoopEmojis(JSON.parse(String(value || '[]')));
    } catch {
        return [];
    }
}

export async function getRecentBoopEmojis(): Promise<BoopEmojiChoice[]> {
    const value = await configRepository.getString(
        RECENT_BOOP_EMOJIS_CONFIG_KEY,
        '[]'
    );
    return parseRecentBoopEmojis(value);
}

export function recordRecentBoopEmoji(choice: BoopEmojiChoice): Promise<void> {
    const write = writeQueue.then(async () => {
        const current = await getRecentBoopEmojis();
        const next = normalizeRecentBoopEmojis([
            choice,
            ...current.filter((known) => !isSameBoopEmoji(known, choice))
        ]);
        await configRepository.setString(
            RECENT_BOOP_EMOJIS_CONFIG_KEY,
            JSON.stringify(next)
        );
    });
    writeQueue = write.catch(() => {});
    return write;
}
