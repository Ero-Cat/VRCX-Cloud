/**
 * Word-level bio diff with CJK awareness, ported from VRCX-jirai's
 * `formatDifference` (MIT): whitespace tokens stay whole, CJK characters
 * tokenize individually so Chinese text diffs at character level.
 */

export type BioDiffSegmentKind = 'same' | 'added' | 'removed';

export interface BioDiffSegment {
    kind: BioDiffSegmentKind;
    text: string;
}

const CJK_CHAR_RE = /[\u3040-\u30ff\u3400-\u9fff\uac00-\ud7a3\uf900-\ufaff]/;

function isCJKChar(ch: string | undefined): boolean {
    return ch ? CJK_CHAR_RE.test(ch) : false;
}

function tokenize(value: string): string[] {
    return value
        .split(/\s+/)
        .flatMap((chunk) => chunk.split(/(\n)/))
        .flatMap((chunk) => {
            if (!chunk || chunk === '\n') {
                return chunk ? [chunk] : [];
            }
            return chunk
                .split(new RegExp(`(${CJK_CHAR_RE.source})`))
                .filter(Boolean);
        });
}

function joinTokens(tokens: string[]): string {
    if (tokens.length === 0) {
        return '';
    }
    let result = tokens[0];
    for (let index = 1; index < tokens.length; index += 1) {
        const previous = tokens[index - 1];
        const current = tokens[index];
        const previousLast = previous.length
            ? previous[previous.length - 1]
            : '';
        const currentFirst = current.length ? current[0] : '';
        if (isCJKChar(previousLast) || isCJKChar(currentFirst)) {
            result += current;
        } else {
            result += ` ${current}`;
        }
    }
    return result;
}

interface Match {
    oldStart: number;
    newStart: number;
    size: number;
}

function findLongestMatch(
    oldWords: string[],
    newWords: string[],
    oldStart: number,
    oldEnd: number,
    newStart: number,
    newEnd: number
): Match {
    let bestOldStart = oldStart;
    let bestNewStart = newStart;
    let bestSize = 0;

    const lookup = new Map<string, number[]>();
    for (let index = oldStart; index < oldEnd; index += 1) {
        const word = oldWords[index];
        const positions = lookup.get(word);
        if (positions) {
            positions.push(index);
        } else {
            lookup.set(word, [index]);
        }
    }

    for (let index = newStart; index < newEnd; index += 1) {
        const word = newWords[index];
        const positions = lookup.get(word);
        if (!positions) {
            continue;
        }
        for (const oldIndex of positions) {
            let size = 0;
            while (
                oldIndex + size < oldEnd &&
                index + size < newEnd &&
                oldWords[oldIndex + size] === newWords[index + size]
            ) {
                size += 1;
            }
            if (size > bestSize) {
                bestOldStart = oldIndex;
                bestNewStart = index;
                bestSize = size;
            }
        }
    }
    return { oldStart: bestOldStart, newStart: bestNewStart, size: bestSize };
}

function buildSegments(
    oldWords: string[],
    newWords: string[],
    oldStart: number,
    oldEnd: number,
    newStart: number,
    newEnd: number,
    segments: BioDiffSegment[]
): void {
    const match = findLongestMatch(
        oldWords,
        newWords,
        oldStart,
        oldEnd,
        newStart,
        newEnd
    );

    if (match.size > 0) {
        if (oldStart < match.oldStart || newStart < match.newStart) {
            buildSegments(
                oldWords,
                newWords,
                oldStart,
                match.oldStart,
                newStart,
                match.newStart,
                segments
            );
        }
        segments.push({
            kind: 'same',
            text: joinTokens(
                newWords.slice(match.newStart, match.newStart + match.size)
            )
        });
        if (
            match.oldStart + match.size < oldEnd ||
            match.newStart + match.size < newEnd
        ) {
            buildSegments(
                oldWords,
                newWords,
                match.oldStart + match.size,
                oldEnd,
                match.newStart + match.size,
                newEnd,
                segments
            );
        }
        return;
    }

    if (oldStart < oldEnd) {
        segments.push({
            kind: 'removed',
            text: joinTokens(oldWords.slice(oldStart, oldEnd))
        });
    }
    if (newStart < newEnd) {
        segments.push({
            kind: 'added',
            text: joinTokens(newWords.slice(newStart, newEnd))
        });
    }
}

export function computeBioDiff(
    oldString: string | null | undefined,
    newString: string | null | undefined
): BioDiffSegment[] {
    const oldWords = tokenize(String(oldString ?? ''));
    const newWords = tokenize(String(newString ?? ''));
    const segments: BioDiffSegment[] = [];
    buildSegments(
        oldWords,
        newWords,
        0,
        oldWords.length,
        0,
        newWords.length,
        segments
    );
    return segments;
}

export function hasBioDiffChanges(segments: BioDiffSegment[]): boolean {
    return segments.some((segment) => segment.kind !== 'same');
}
