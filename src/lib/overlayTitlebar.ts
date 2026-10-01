const APP_TITLE_BAR_SELECTOR = '[data-app-titlebar="true"]';
const WINDOW_ACTION_GRACE_PERIOD_MS = 500;

let pendingWindowAction = false;
let clearPendingWindowActionTimer: ReturnType<typeof setTimeout> | undefined;

type OverlayCloseEventDetails = {
    reason: string;
    event: Event;
    cancel: () => void;
};

function isAppTitleBarTarget(target: EventTarget | null) {
    return (
        target instanceof Element &&
        Boolean(target.closest(APP_TITLE_BAR_SELECTOR))
    );
}

function markAppTitleBarWindowAction() {
    clearPendingWindowAction();
    pendingWindowAction = true;
    // A new gesture must clear protection before Base UI's document capture handlers.
    window.addEventListener('pointerdown', clearPendingWindowAction, true);
    window.addEventListener('pointercancel', clearPendingWindowAction, true);
    window.addEventListener('keydown', clearPendingWindowAction, true);
    clearPendingWindowActionTimer = setTimeout(
        clearPendingWindowAction,
        WINDOW_ACTION_GRACE_PERIOD_MS
    );
}

function clearPendingWindowAction() {
    pendingWindowAction = false;
    window.removeEventListener('pointerdown', clearPendingWindowAction, true);
    window.removeEventListener('pointercancel', clearPendingWindowAction, true);
    window.removeEventListener('keydown', clearPendingWindowAction, true);
    if (clearPendingWindowActionTimer) {
        clearTimeout(clearPendingWindowActionTimer);
        clearPendingWindowActionTimer = undefined;
    }
}

function preserveAppTitleBarOnOpenChange(
    open: boolean,
    eventDetails: OverlayCloseEventDetails
) {
    const isTitleBarTarget = isAppTitleBarTarget(eventDetails.event.target);
    if (
        !open &&
        eventDetails.reason === 'outside-press' &&
        (isTitleBarTarget || pendingWindowAction)
    ) {
        eventDetails.cancel();
        if (!isTitleBarTarget) {
            clearPendingWindowAction();
        }
        return true;
    }
    return false;
}

export { markAppTitleBarWindowAction, preserveAppTitleBarOnOpenChange };
