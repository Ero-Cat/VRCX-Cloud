import { tauriEvents } from './events';
import * as webviewSurface from './webview';

type RuntimeEvents = typeof tauriEvents;

export interface PlatformClient {
    events: RuntimeEvents;
    webview: typeof webviewSurface;
}

/**
 * Historical name kept so generated/consumer imports stay stable.
 */
export const tauriClient: PlatformClient = Object.freeze({
    events: tauriEvents,
    webview: webviewSurface
});
