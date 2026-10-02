export type BioDiffLineKind = 'same' | 'added' | 'removed';

export type BioDiffLine = {
    kind: BioDiffLineKind;
    text: string;
};

function splitLines(value: string | null | undefined): string[] {
    if (!value) {
        return [];
    }
    return String(value).replace(/\r\n/g, '\n').split('\n');
}

/**
 * Line diff between two bio snapshots in git-diff spirit: removed lines
 * (previous bio), added lines (next bio), unchanged context. Uses an LCS
 * table; friend bios are short so the quadratic cost is irrelevant.
 */
export function computeBioLineDiff(
    previousBio: string | null | undefined,
    nextBio: string | null | undefined
): BioDiffLine[] {
    const previousLines = splitLines(previousBio);
    const nextLines = splitLines(nextBio);
    const rows = previousLines.length;
    const columns = nextLines.length;

    // lcs[row][col] = LCS length of previousLines[row..] and nextLines[col..]
    const lcs: number[][] = Array.from({ length: rows + 1 }, () =>
        Array.from<number>({ length: columns + 1 }).fill(0)
    );
    for (let row = rows - 1; row >= 0; row -= 1) {
        for (let col = columns - 1; col >= 0; col -= 1) {
            lcs[row][col] =
                previousLines[row] === nextLines[col]
                    ? lcs[row + 1][col + 1] + 1
                    : Math.max(lcs[row + 1][col], lcs[row][col + 1]);
        }
    }

    const lines: BioDiffLine[] = [];
    let row = 0;
    let col = 0;
    while (row < rows && col < columns) {
        if (previousLines[row] === nextLines[col]) {
            lines.push({ kind: 'same', text: nextLines[col] });
            row += 1;
            col += 1;
        } else if (lcs[row + 1][col] >= lcs[row][col + 1]) {
            lines.push({ kind: 'removed', text: previousLines[row] });
            row += 1;
        } else {
            lines.push({ kind: 'added', text: nextLines[col] });
            col += 1;
        }
    }
    while (row < rows) {
        lines.push({ kind: 'removed', text: previousLines[row] });
        row += 1;
    }
    while (col < columns) {
        lines.push({ kind: 'added', text: nextLines[col] });
        col += 1;
    }
    return lines;
}

/** Collapse runs of unchanged lines longer than the context window. */
export function collapseBioDiffContext(
    lines: BioDiffLine[],
    contextLines = 3
): BioDiffLine[] {
    const keep = Array.from<boolean>({ length: lines.length }).fill(false);
    lines.forEach((line, index) => {
        if (line.kind === 'same') {
            return;
        }
        const from = Math.max(0, index - contextLines);
        const to = Math.min(lines.length - 1, index + contextLines);
        for (let cursor = from; cursor <= to; cursor += 1) {
            keep[cursor] = true;
        }
    });
    const output: BioDiffLine[] = [];
    let skipping = false;
    lines.forEach((line, index) => {
        if (keep[index]) {
            output.push(line);
            skipping = false;
        } else if (!skipping) {
            output.push({ kind: 'same', text: '…' });
            skipping = true;
        }
    });
    return output;
}

export function hasBioChanges(lines: BioDiffLine[]): boolean {
    return lines.some((line) => line.kind !== 'same');
}
