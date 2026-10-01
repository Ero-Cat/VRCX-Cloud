type EmojiFrameLayout = {
    frameCount: number;
    framesPerLine: number;
    frameSize: number;
};

function getEmojiFrameLayout(frameCount: number): EmojiFrameLayout {
    const normalizedFrameCount = Math.min(
        64,
        Math.max(1, Number.isFinite(frameCount) ? Math.trunc(frameCount) : 1)
    );
    let framesPerLine = 2;
    if (normalizedFrameCount > 4) framesPerLine = 4;
    if (normalizedFrameCount > 16) framesPerLine = 8;
    const frameSize = 1024 / framesPerLine;
    return {
        frameCount: normalizedFrameCount,
        framesPerLine,
        frameSize
    };
}

function getEmojiAnimationName(frameCount: number): string {
    return `animated-emoji-${getEmojiFrameLayout(frameCount).frameCount}`;
}

function buildEmojiKeyframes(frameCount: number): string {
    const { frameCount: normalizedFrameCount, framesPerLine } =
        getEmojiFrameLayout(frameCount);
    const maxFrameIndex = framesPerLine - 1;
    const rules: string[] = [];
    for (let index = 0; index < normalizedFrameCount; index += 1) {
        const percent = (index / normalizedFrameCount) * 100;
        const column = index % framesPerLine;
        const row = Math.floor(index / framesPerLine);
        const x = maxFrameIndex > 0 ? (column / maxFrameIndex) * 100 : 0;
        const y = maxFrameIndex > 0 ? (row / maxFrameIndex) * 100 : 0;
        rules.push(`${percent}%{background-position:${x}% ${y}%;}`);
    }
    return rules.join('');
}

export { getEmojiFrameLayout, getEmojiAnimationName, buildEmojiKeyframes };
