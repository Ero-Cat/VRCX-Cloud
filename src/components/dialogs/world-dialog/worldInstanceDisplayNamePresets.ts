export const INSTANCE_DIALOG_DISPLAY_NAME_KEY = 'instanceDialogDisplayName';
export const INSTANCE_DIALOG_DISPLAY_NAME_PRESETS_KEY =
    'instanceDialogDisplayNamePresets';

const INSTANCE_DIALOG_DISPLAY_NAME_PRESET_LIMIT = 10;

export function normalizeInstanceDialogDisplayName(value: unknown) {
    return String(value ?? '').trim();
}

export function normalizeInstanceDialogDisplayNamePresets(
    values: unknown,
    fallback: string = ''
) {
    const next: string[] = [];
    const seen = new Set<string>();

    function append(value: unknown) {
        const normalized = normalizeInstanceDialogDisplayName(value);
        if (!normalized || seen.has(normalized)) {
            return;
        }
        seen.add(normalized);
        next.push(normalized);
    }

    append(fallback);
    if (Array.isArray(values)) {
        for (const value of values) {
            append(value);
        }
    }

    return next.slice(0, INSTANCE_DIALOG_DISPLAY_NAME_PRESET_LIMIT);
}

export function prependInstanceDialogDisplayNamePreset(
    values: unknown,
    value: string
) {
    const normalized = normalizeInstanceDialogDisplayName(value);
    if (!normalized) {
        return normalizeInstanceDialogDisplayNamePresets(values);
    }
    return normalizeInstanceDialogDisplayNamePresets(values, normalized);
}
