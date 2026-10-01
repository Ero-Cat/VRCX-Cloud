export function createDensityPreset<
    C extends Readonly<Record<string, unknown>>
>(defaultValue: keyof C & string, configs: C) {
    const values = new Set(Object.keys(configs));

    function isDensityValue(value: string): value is keyof C & string {
        return values.has(value);
    }

    function sanitize(value?: unknown): keyof C & string {
        const normalizedValue = typeof value === 'string' ? value.trim() : '';
        return isDensityValue(normalizedValue) ? normalizedValue : defaultValue;
    }

    function getConfig(value?: unknown): C[keyof C & string] {
        return configs[sanitize(value)];
    }

    return { sanitize, getConfig };
}
