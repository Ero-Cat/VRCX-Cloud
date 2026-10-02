import { normalizePlatformError } from './errors';
import { ensureWebEventConnection } from './webTransport';

export type TauriEventHandler<TPayload = unknown> = (payload: TPayload) => void;

type UnlistenFn = () => void;

const listeners = new Map<string, Set<TauriEventHandler>>();

function getBucket(name: string): Set<TauriEventHandler> {
    let bucket = listeners.get(name);
    if (!bucket) {
        bucket = new Set();
        listeners.set(name, bucket);
    }
    return bucket;
}

function dispatch(name: string, payload: unknown): void {
    const bucket = listeners.get(name);
    if (!bucket || bucket.size === 0) {
        return;
    }

    for (const handler of bucket) {
        try {
            handler(payload);
        } catch (error) {
            console.error(`Error in runtime event handler for ${name}:`, error);
        }
    }
}

async function ensureWebSubscription(name: string): Promise<UnlistenFn> {
    try {
        ensureWebEventConnection(dispatch);
    } catch (error) {
        throw normalizePlatformError(
            error,
            `Unable to subscribe to runtime event: ${name}`
        );
    }
    return () => undefined;
}

export async function onTauriEvent(
    name: string,
    handler: TauriEventHandler
): Promise<() => void> {
    const handlerBucket = getBucket(name);
    handlerBucket.add(handler);
    try {
        await ensureWebSubscription(name);
    } catch (error) {
        if (listeners.get(name) === handlerBucket) {
            offTauriEvent(name, handler);
        }
        throw error;
    }

    return () => {
        if (listeners.get(name) === handlerBucket) {
            offTauriEvent(name, handler);
        }
    };
}

async function subscribeWebEvent<TPayload = unknown>(
    name: string,
    handler: TauriEventHandler<TPayload>
): Promise<() => void> {
    const eventHandler = handler as TauriEventHandler;
    const handlerBucket = getBucket(name);
    handlerBucket.add(eventHandler);
    try {
        await ensureWebSubscription(name);
    } catch (error) {
        if (listeners.get(name) === handlerBucket) {
            offTauriEvent(name, eventHandler);
        }
        throw error;
    }

    return () => {
        if (listeners.get(name) === handlerBucket) {
            offTauriEvent(name, eventHandler);
        }
    };
}

export function offTauriEvent(name: string, handler: TauriEventHandler): void {
    listeners.get(name)?.delete(handler);
    if (listeners.get(name)?.size === 0) {
        listeners.delete(name);
    }
}

export const tauriEvents = {
    on: onTauriEvent,
    off: offTauriEvent,
    subscribe: subscribeWebEvent,
    emit: () => {},
    clear: () => {
        listeners.clear();
    }
};
