// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
    markAppTitleBarWindowAction,
    preserveAppTitleBarOnOpenChange
} from './overlayTitlebar';

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
});

describe('preserveAppTitleBarOnOpenChange', () => {
    it('keeps an overlay open when window restoration changes the event target', () => {
        const cancel = vi.fn();
        markAppTitleBarWindowAction();
        expect(
            preserveAppTitleBarOnOpenChange(false, {
                reason: 'outside-press',
                event: new Event('click'),
                cancel
            })
        ).toBe(true);
        expect(cancel).toHaveBeenCalledOnce();
    });

    it.each(['pointerdown', 'keydown'])(
        'allows a new interaction after %s during the grace period',
        (type) => {
            const cancel = vi.fn();
            markAppTitleBarWindowAction();
            document.body.dispatchEvent(new Event(type, { bubbles: true }));
            expect(
                preserveAppTitleBarOnOpenChange(false, {
                    reason: 'outside-press',
                    event: new Event('click'),
                    cancel
                })
            ).toBe(false);
            expect(cancel).not.toHaveBeenCalled();
        }
    );

    it('expires protection when there is no follow-up event', () => {
        const cancel = vi.fn();
        markAppTitleBarWindowAction();
        vi.advanceTimersByTime(500);
        expect(
            preserveAppTitleBarOnOpenChange(false, {
                reason: 'outside-press',
                event: new Event('click'),
                cancel
            })
        ).toBe(false);
        expect(cancel).not.toHaveBeenCalled();
    });
});
