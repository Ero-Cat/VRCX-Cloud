export const TRUST_COLOR_DEFAULTS = Object.freeze({
    untrusted: '#CCCCCC',
    basic: '#1778FF',
    known: '#2BCF5C',
    trusted: '#FF7B42',
    veteran: '#B18FFF',
    vip: '#FF2626',
    troll: '#782F2F'
});

export const TRUST_COLOR_ENTRIES = Object.freeze([
    {
        key: 'untrusted',
        className: 'x-tag-untrusted',
        labelKey: 'view.settings.appearance.user_colors.trust_levels.visitor',
        presets: Object.freeze(['#CCCCCC'])
    },
    {
        key: 'basic',
        className: 'x-tag-basic',
        labelKey: 'view.settings.appearance.user_colors.trust_levels.new_user',
        presets: Object.freeze(['#1778ff'])
    },
    {
        key: 'known',
        className: 'x-tag-known',
        labelKey: 'view.settings.appearance.user_colors.trust_levels.user',
        presets: Object.freeze(['#2bcf5c'])
    },
    {
        key: 'trusted',
        className: 'x-tag-trusted',
        labelKey:
            'view.settings.appearance.user_colors.trust_levels.known_user',
        presets: Object.freeze(['#ff7b42'])
    },
    {
        key: 'veteran',
        className: 'x-tag-veteran',
        labelKey:
            'view.settings.appearance.user_colors.trust_levels.trusted_user',
        presets: Object.freeze([
            '#b18fff',
            '#8143e6',
            '#ff69b4',
            '#b52626',
            '#ffd000',
            '#abcdef'
        ])
    },
    {
        key: 'vip',
        className: 'x-tag-vip',
        labelKey:
            'view.settings.appearance.user_colors.trust_levels.vrchat_team',
        presets: Object.freeze(['#ff2626'])
    },
    {
        key: 'troll',
        className: 'x-tag-troll',
        labelKey: 'view.settings.appearance.user_colors.trust_levels.nuisance',
        presets: Object.freeze(['#782f2f'])
    }
] as const);
