export type {
    NormalizedRelease,
    UpdateDownloadProgress
} from './update-service/types';

export {
    fetchBranchReleases,
    fetchLatestBranchRelease
} from './update-service/github';
export { formatReleaseDisplayVersion } from '@/shared/utils/releaseVersion';
