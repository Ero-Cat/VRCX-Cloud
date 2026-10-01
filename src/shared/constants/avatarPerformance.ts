// Official rank maximums, checked 2026-09-07. Keep this table in sync with:
// https://creators.vrchat.com/avatars/avatar-performance-ranking-system/
// The displayed denominator is Poor's maximum, not an upload limit or target.
export const PERFORMANCE_RANKS = [
    'Excellent',
    'Good',
    'Medium',
    'Poor',
    'VeryPoor'
] as const;
export type PerformanceRank = (typeof PERFORMANCE_RANKS)[number];
export type PerformancePlatform = 'pc' | 'android' | 'ios';
export type PerformanceLimits = readonly [number, number, number, number];

export const PC_PERFORMANCE_LIMITS: Readonly<
    Record<string, PerformanceLimits | undefined>
> = {
    totalPolygons: [32000, 70000, 70000, 70000],
    totalTextureUsage: [40, 75, 110, 150],
    skinnedMeshCount: [1, 2, 8, 16],
    meshCount: [4, 8, 16, 24],
    materialSlotsUsed: [4, 8, 16, 32],
    physBoneComponentCount: [4, 8, 16, 32],
    physBoneTransformCount: [16, 64, 128, 256],
    physBoneColliderCount: [4, 8, 16, 32],
    physBoneCollisionCheckCount: [32, 128, 256, 512],
    contactCount: [8, 16, 24, 32],
    constraintCount: [100, 250, 300, 350],
    constraintDepth: [20, 50, 80, 100],
    animatorCount: [1, 4, 16, 32],
    boneCount: [75, 150, 256, 400],
    lightCount: [0, 0, 0, 1],
    particleSystemCount: [0, 4, 8, 16],
    totalMaxParticles: [0, 300, 1000, 2500],
    meshParticleMaxPolygons: [0, 1000, 2000, 5000],
    particleTrailsEnabled: [0, 0, 1, 1],
    particleCollisionEnabled: [0, 0, 1, 1],
    trailRendererCount: [1, 2, 4, 8],
    lineRendererCount: [1, 2, 4, 8],
    raycastCount: [1, 4, 8, 15],
    clothCount: [0, 1, 1, 1],
    totalClothVertices: [0, 50, 100, 200],
    physicsColliders: [0, 1, 8, 8],
    physicsRigidbodies: [0, 1, 8, 8],
    audioSourceCount: [1, 4, 8, 8]
};

export const MOBILE_PERFORMANCE_LIMITS: Readonly<
    Record<string, PerformanceLimits | undefined>
> = {
    totalPolygons: [7500, 10000, 15000, 20000],
    totalTextureUsage: [10, 18, 25, 40],
    skinnedMeshCount: [1, 1, 2, 2],
    meshCount: [1, 1, 2, 2],
    materialSlotsUsed: [1, 1, 2, 4],
    animatorCount: [1, 1, 1, 2],
    boneCount: [75, 90, 150, 150],
    physBoneComponentCount: [0, 4, 6, 8],
    physBoneTransformCount: [0, 16, 32, 64],
    physBoneColliderCount: [0, 4, 8, 16],
    physBoneCollisionCheckCount: [0, 16, 32, 64],
    contactCount: [2, 4, 8, 16],
    constraintCount: [30, 60, 120, 150],
    constraintDepth: [5, 15, 35, 50],
    particleSystemCount: [0, 0, 0, 2],
    totalMaxParticles: [0, 0, 0, 200],
    meshParticleMaxPolygons: [0, 0, 0, 400],
    particleTrailsEnabled: [0, 0, 0, 1],
    particleCollisionEnabled: [0, 0, 0, 1],
    trailRendererCount: [0, 0, 0, 1],
    lineRendererCount: [0, 0, 0, 1],
    raycastCount: [1, 2, 4, 8]
};

export const MOBILE_REMOVED_STATS: ReadonlySet<string> = new Set([
    'lightCount',
    'clothCount',
    'totalClothVertices',
    'physicsColliders',
    'physicsRigidbodies',
    'audioSourceCount'
]);
export const PERFORMANCE_BOUNDS_LIMITS = [
    [2.5, 2.5, 2.5],
    [4, 4, 4],
    [5, 6, 5],
    [5, 6, 5]
];

export const PERFORMANCE_TEXTURE_BYTES_PER_MIB = 1048576;
