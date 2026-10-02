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
    | 'unauthorized'
    | 'badRequest'
    | 'unsupportedOnWeb'
    | 'commandFailed';

export class WebCommandError extends Error {
    readonly code: WebErrorCode;

    constructor(code: WebErrorCode, message: string) {
        super(message);
        this.name = 'WebCommandError';
        this.code = code;
    }
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

    if (response.status === 401) {
        throw new WebCommandError('unauthorized', 'Web session expired');
    }
    if (!envelope.ok) {
        const code = (envelope.code ?? 'commandFailed') as WebErrorCode;
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

export interface WebAuthStatus {
    authEnabled: boolean;
    sessionValid: boolean;
}

export async function webAuthStatus(): Promise<WebAuthStatus> {
    const [statusResponse, sessionResponse] = await Promise.all([
        fetch('/api/auth/status', { credentials: 'same-origin' }),
        fetch('/api/auth/session', { credentials: 'same-origin' })
    ]);
    const status = (await statusResponse.json()) as { authEnabled?: boolean };
    const session = (await sessionResponse.json()) as { valid?: boolean };
    return {
        authEnabled: status.authEnabled === true,
        sessionValid: session.valid !== false
    };
}

export async function webLogin(password: string): Promise<boolean> {
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ password })
    });
    return response.ok;
}

export async function webLogout(): Promise<void> {
    await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'same-origin'
    });
}
