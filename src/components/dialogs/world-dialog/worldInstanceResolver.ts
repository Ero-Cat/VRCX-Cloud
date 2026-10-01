import vrchatInstanceRepository from '@/repositories/vrchatInstanceRepository';
import { parseLocation } from '@/shared/utils/location';
import { isRecord } from '@/shared/utils/record';

import {
    buildCreatedInstanceDetails,
    type CreatedInstanceFallback
} from './worldInstances';

export async function resolveCreatedInstanceDetails(
    location: string,
    instance: unknown,
    fallback: CreatedInstanceFallback = {}
) {
    const parsedLocation = parseLocation(location);
    const source = isRecord(instance) ? instance : {};
    if (
        !parsedLocation.worldId ||
        !parsedLocation.instanceId ||
        source.shortName
    ) {
        return buildCreatedInstanceDetails(location, instance, fallback);
    }
    try {
        const response = await vrchatInstanceRepository.getInstanceShortName({
            worldId: parsedLocation.worldId,
            instanceId: parsedLocation.instanceId
        });
        return buildCreatedInstanceDetails(
            location,
            {
                ...source,
                shortName: response.json?.shortName,
                secureName: response.json?.secureName
            },
            fallback
        );
    } catch {
        return buildCreatedInstanceDetails(location, instance, fallback);
    }
}
