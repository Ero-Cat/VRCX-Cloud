import {
    TRUST_COLOR_DEFAULTS,
    TRUST_COLOR_ENTRIES
} from '@/shared/constants/trustColors';

const HEX_COLOR_PATTERN = /^#[0-9a-f]{6}$/i;

type TrustColorKey = keyof typeof TRUST_COLOR_DEFAULTS;
type TrustColorMap = Record<TrustColorKey, string>;
type TrustColorUser = Record<string, unknown>;

function parseTrustColorSource(value: unknown): Record<string, unknown> {
    if (value && typeof value === 'object') {
        return Object.fromEntries(Object.entries(value));
    }
    if (typeof value !== 'string' || !value.trim()) {
        return {};
    }
    try {
        const parsed = JSON.parse(value);
        return parsed && typeof parsed === 'object' ? parsed : {};
    } catch {
        return {};
    }
}

function isTrustColorKey(value: string): value is TrustColorKey {
    return Object.prototype.hasOwnProperty.call(TRUST_COLOR_DEFAULTS, value);
}

export function normalizeTrustColors(value: unknown): TrustColorMap {
    const source = parseTrustColorSource(value);
    const normalized: TrustColorMap = { ...TRUST_COLOR_DEFAULTS };
    for (const { key } of TRUST_COLOR_ENTRIES) {
        const color = String(source[key] || '').trim();
        normalized[key] = HEX_COLOR_PATTERN.test(color)
            ? color.toUpperCase()
            : TRUST_COLOR_DEFAULTS[key];
    }
    return normalized;
}

export function isValidTrustColor(value: unknown) {
    return HEX_COLOR_PATTERN.test(String(value || '').trim());
}

export function resolveTrustColorKey(user: unknown): TrustColorKey {
    const source =
        user && (typeof user === 'object' || typeof user === 'function')
            ? (Object.fromEntries(
                  Object.entries(user)
              ) satisfies TrustColorUser)
            : {};
    if (source.$isModerator) {
        return 'vip';
    }
    if (source.$isTroll || source.$isProbableTroll) {
        return 'troll';
    }
    const classKey = String(
        source.$trustClass || source.trustClass || ''
    ).replace(/^x-tag-/, '');
    return isTrustColorKey(classKey) ? classKey : 'untrusted';
}

export function getTrustColor(
    user: unknown,
    trustColors: unknown = TRUST_COLOR_DEFAULTS
) {
    const normalized = normalizeTrustColors(trustColors);
    return normalized[resolveTrustColorKey(user)] || normalized.untrusted;
}

export type { TrustColorKey, TrustColorMap };
