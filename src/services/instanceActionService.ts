import {
    commands,
    type InstanceLaunchMode,
    type InstanceLaunchOutcome
} from '@/platform/tauri/bindings';
import { normalizeString } from '@/shared/utils/string';

function failedReason(outcome: InstanceLaunchOutcome): string {
    return outcome.status === 'failed'
        ? outcome.reason
        : 'VRChat action failed.';
}

async function runJoinAction({
    location,
    mode,
    shortName = ''
}: {
    location: string;
    mode: InstanceLaunchMode;
    shortName?: string;
}): Promise<InstanceLaunchOutcome> {
    return commands.appVrchatInstanceJoin({
        location: normalizeString(location),
        shortName: normalizeString(shortName),
        mode
    });
}

async function openInstanceInGame(
    location: string,
    shortName: string = ''
): Promise<boolean> {
    try {
        const outcome = await runJoinAction({
            location,
            shortName,
            mode: 'openOnly'
        });
        return outcome.status === 'opened';
    } catch (error) {
        console.warn('Failed to open VRChat launch URL through IPC:', error);
        return false;
    }
}

async function sendSelfInviteToInstance(
    location: string,
    shortName: string = ''
): Promise<void> {
    const outcome = await runJoinAction({
        location,
        shortName,
        mode: 'selfInviteOnly'
    });
    if (outcome.status !== 'selfInvited') {
        throw new Error(failedReason(outcome));
    }
}

async function joinInstanceWithFallback(
    location: string,
    shortName: string = ''
): Promise<InstanceLaunchOutcome> {
    return runJoinAction({
        location,
        shortName,
        mode: 'auto'
    });
}

export {
    joinInstanceWithFallback,
    openInstanceInGame,
    sendSelfInviteToInstance
};
