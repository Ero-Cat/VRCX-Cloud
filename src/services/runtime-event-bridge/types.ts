import type { GroupInstanceRecord } from '@/domain/entities/group';
import type {
    BackendRuntimeEventPayloadMap,
    BackendRuntimeSnapshot
} from '@/platform/tauri/bindings';

import type {
    RealtimeCurrentUserProjectionPayload,
    RealtimeEntryCorrectionPayload,
    RealtimeFeedProjectionPayload,
    RealtimeFriendProjectionPayload,
    RealtimeInstanceClosedProjectionPayload,
    RealtimeNotificationProjectionPayload,
    RealtimeUserProjectionPayload
} from './realtimeProjectionTypes';

type RealtimePayloadOverrides = {
    realtimeCurrentUserProjection: RealtimeCurrentUserProjectionPayload;
    realtimeEntryCorrection: RealtimeEntryCorrectionPayload;
    realtimeFeedProjection: RealtimeFeedProjectionPayload;
    realtimeFriendProjection: RealtimeFriendProjectionPayload;
    realtimeInstanceClosedProjection: RealtimeInstanceClosedProjectionPayload;
    realtimeNotificationProjection: RealtimeNotificationProjectionPayload;
    realtimeUserProjection: RealtimeUserProjectionPayload;
};

type RuntimeGroupInstancesProjectionPayload = Omit<
    BackendRuntimeEventPayloadMap['runtimeGroupInstancesProjection'],
    'instances'
> & {
    instances?: GroupInstanceRecord[] | null;
};

type RuntimePayloadOverrides = RealtimePayloadOverrides & {
    runtimeGroupInstancesProjection: RuntimeGroupInstancesProjectionPayload;
};

export type RuntimeEventPayloadMap = Omit<
    BackendRuntimeEventPayloadMap,
    keyof RuntimePayloadOverrides
> &
    RuntimePayloadOverrides & {
        browserFocus: null;
    };

export type RuntimeEventName = keyof RuntimeEventPayloadMap;

export type RuntimeEvent<Name extends RuntimeEventName = RuntimeEventName> = {
    [EventName in Name]: {
        name: EventName;
        payload: RuntimeEventPayloadMap[EventName];
    };
}[Name];

export type FavoritesChangedEventPayload =
    RuntimeEventPayloadMap['favoritesChanged'];

export type RuntimeGroupInstancesProjection =
    RuntimeEventPayloadMap['runtimeGroupInstancesProjection'];

export type RuntimeSnapshotPayload = BackendRuntimeSnapshot | null;
