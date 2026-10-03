import { useAdminAuthStore } from '@/state/adminAuthStore';

/**
 * Web transport for the self-hosted server build.
 *
 * When the frontend runs in a plain browser (no Tauri internals), every
 * command goes to `POST /api/invoke` and runtime events arrive over the
 * `GET /api/events` WebSocket, speaking the same command names, kwargs
 * and `{event, payload}` frames the desktop build uses.
 */

export function isWebPlatform(): boolean {
    // Browser-only build.
    return true;
}

export type WebErrorCode =
    | 'badRequest'
    | 'unsupportedOnWeb'
    | 'commandFailed'
    | 'adminAuthRequired';

export class WebCommandError extends Error {
    readonly code: WebErrorCode;

    constructor(code: WebErrorCode, message: string) {
        super(message);
        this.name = 'WebCommandError';
        this.code = code;
    }
}

/**
 * Whether an error (possibly wrapped by `normalizePlatformError`) is the
 * admin browser gate rejecting the request. Callers that surface command
 * failures to the user should stay quiet in this case: the unlock dialog
 * is already up and the app reboots after a successful unlock.
 */
export function isAdminGateError(error: unknown): boolean {
    let current: unknown = error;
    for (let depth = 0; depth < 8 && current; depth += 1) {
        if (
            current instanceof WebCommandError &&
            current.code === 'adminAuthRequired'
        ) {
            return true;
        }
        current = (current as { cause?: unknown }).cause;
    }
    return false;
}

interface InvokeEnvelope {
    ok?: boolean;
    result?: unknown;
    code?: string;
    message?: string;
}

export async function webInvoke<TReturn = unknown>(
    command: string,
    args?: Record<string, unknown>
): Promise<TReturn> {
    let response: Response;
    try {
        response = await fetch('/api/invoke', {
            method: 'POST',
            credentials: 'same-origin',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ cmd: command, args: args ?? {} })
        });
    } catch {
        throw new WebCommandError(
            'commandFailed',
            `Web command transport failed: ${command}`
        );
    }

    let envelope: InvokeEnvelope;
    try {
        envelope = (await response.json()) as InvokeEnvelope;
    } catch {
        throw new WebCommandError(
            'commandFailed',
            `Web command returned an invalid response: ${command}`
        );
    }

    if (!envelope.ok) {
        const code = (envelope.code ?? 'commandFailed') as WebErrorCode;
        if (code === 'adminAuthRequired') {
            // The server's admin gate rejected this browser; surface the
            // unlock dialog. This command stays failed — the app reboots
            // with the unlock cookie after a successful unlock.
            useAdminAuthStore.getState().markLocked();
        }
        throw new WebCommandError(code, envelope.message ?? command);
    }
    return envelope.result as TReturn;
}

type WebEventDispatcher = (event: string, payload: unknown) => void;

let webSocket: WebSocket | null = null;
let webDispatcher: WebEventDispatcher | null = null;
let webReconnectDelayMs = 500;
let webReconnectTimer: ReturnType<typeof setTimeout> | null = null;

export function ensureWebEventConnection(dispatch: WebEventDispatcher): void {
    webDispatcher = dispatch;
    if (
        webSocket &&
        (webSocket.readyState === WebSocket.OPEN ||
            webSocket.readyState === WebSocket.CONNECTING)
    ) {
        return;
    }
    if (typeof window === 'undefined') {
        return;
    }
    if (useAdminAuthStore.getState().phase === 'locked') {
        // The admin gate rejects the upgrade until this browser
        // unlocks; the page reboots after unlocking and reconnects.
        return;
    }

    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const socket = new WebSocket(
        `${protocol}://${window.location.host}/api/events`
    );
    webSocket = socket;

    socket.addEventListener('open', () => {
        webReconnectDelayMs = 500;
    });
    socket.addEventListener('message', (message) => {
        try {
            const frame = JSON.parse(String(message.data)) as {
                event?: string;
                payload?: unknown;
            };
            if (frame.event) {
                webDispatcher?.(frame.event, frame.payload);
            }
        } catch {
            // Ignore malformed frames; snapshots re-hydrate state.
        }
    });
    const scheduleReconnect = () => {
        if (webSocket !== socket) {
            return;
        }
        webSocket = null;
        if (webReconnectTimer || !webDispatcher) {
            return;
        }
        webReconnectTimer = setTimeout(() => {
            webReconnectTimer = null;
            if (webDispatcher) {
                ensureWebEventConnection(webDispatcher);
            }
        }, webReconnectDelayMs);
        webReconnectDelayMs = Math.min(webReconnectDelayMs * 2, 30_000);
    };
    socket.addEventListener('close', scheduleReconnect);
    socket.addEventListener('error', scheduleReconnect);
}
