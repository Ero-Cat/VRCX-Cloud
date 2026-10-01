const HOVER_OPEN_SUPPRESSION_BUFFER_MS = 100;

export function getHoverOpenSuppressionDeadline(
    nowMs: number,
    openDelayMs: number
) {
    return nowMs + Math.max(0, openDelayMs) + HOVER_OPEN_SUPPRESSION_BUFFER_MS;
}
