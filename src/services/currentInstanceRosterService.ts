import {
    type CurrentInstanceRosterContext,
    type CurrentInstanceRosterSnapshot
} from '@/domain/instances/currentInstanceRoster';
import currentInstanceRosterRepository, {
    type PlayerListContext as BackendRosterContext
} from '@/repositories/currentInstanceRosterRepository';
import { normalizeString } from '@/shared/utils/string';

interface LoadCurrentInstanceRosterInput {
    currentLocation: string;
}

function normalizeContext(
    context: BackendRosterContext
): CurrentInstanceRosterContext {
    return {
        ...context,
        playerCount: context.playerCount ?? 0
    };
}

export async function loadCurrentInstanceRoster({
    currentLocation
}: LoadCurrentInstanceRosterInput): Promise<CurrentInstanceRosterSnapshot> {
    const normalizedLocation = normalizeString(currentLocation);
    const snapshot =
        await currentInstanceRosterRepository.getCurrentInstanceSnapshot({
            currentLocation: normalizedLocation
        });
    return {
        context: normalizeContext(snapshot.context),
        players: snapshot.players
    };
}
