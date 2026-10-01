import {
    MOBILE_PERFORMANCE_LIMITS,
    MOBILE_REMOVED_STATS,
    PC_PERFORMANCE_LIMITS,
    PERFORMANCE_BOUNDS_LIMITS,
    PERFORMANCE_RANKS,
    PERFORMANCE_TEXTURE_BYTES_PER_MIB,
    type PerformanceLimits,
    type PerformancePlatform,
    type PerformanceRank
} from '@/shared/constants/avatarPerformance';

type StatAssessment = {
    maximum?: number | boolean | number[];
    rank?: PerformanceRank;
    ratio?: number;
    removed?: boolean;
};

function assessBounds(value: unknown): StatAssessment {
    if (
        !Array.isArray(value) ||
        value.length !== 3 ||
        !value.every(
            (axis) =>
                typeof axis === 'number' && Number.isFinite(axis) && axis >= 0
        )
    ) {
        return {};
    }
    const index = PERFORMANCE_BOUNDS_LIMITS.findIndex((limit) =>
        value.every((axis, index) => axis <= limit[index]!)
    );
    const maximum = PERFORMANCE_BOUNDS_LIMITS[3]!;
    return {
        maximum,
        rank: PERFORMANCE_RANKS[index < 0 ? 4 : index],
        ratio: Math.max(...value.map((axis, index) => axis / maximum[index]!))
    };
}

function assessScalar(value: number, limits: PerformanceLimits) {
    const index = limits.findIndex((limit) => value <= limit);
    const maximum = limits[3];
    return {
        maximum,
        rank: PERFORMANCE_RANKS[index < 0 ? 4 : index],
        ratio: maximum > 0 ? value / maximum : Number(value > 0)
    };
}

// Missing/invalid values and undocumented metrics must not look like zero/Excellent.
export function assessPerformanceStat(
    key: string,
    value: unknown,
    platform: PerformancePlatform
): StatAssessment {
    if (platform !== 'pc' && MOBILE_REMOVED_STATS.has(key)) {
        return { removed: true };
    }
    if (key === 'bounds') {
        return assessBounds(value);
    }
    const limits = (
        platform === 'pc' ? PC_PERFORMANCE_LIMITS : MOBILE_PERFORMANCE_LIMITS
    )[key];
    if (!limits) return {};
    if (key === 'particleTrailsEnabled' || key === 'particleCollisionEnabled') {
        if (typeof value !== 'boolean') return {};
        const assessment = assessScalar(Number(value), limits);
        return { ...assessment, maximum: Boolean(assessment.maximum) };
    }
    if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) {
        return {};
    }
    const numeric =
        key === 'totalTextureUsage'
            ? value / PERFORMANCE_TEXTURE_BYTES_PER_MIB
            : value;
    return assessScalar(numeric, limits);
}
