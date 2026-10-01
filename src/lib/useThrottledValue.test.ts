import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createValueThrottle } from './useThrottledValue';

describe('createValueThrottle', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });
    afterEach(() => {
        vi.useRealTimers();
    });

    it('emits the first value immediately on the leading edge', () => {
        const emitted: number[] = [];
        const throttle = createValueThrottle<number>((value) =>
            emitted.push(value)
        );

        throttle.push(1, 1000);

        expect(emitted).toEqual([1]);
    });

    it('coalesces a burst into one trailing emit carrying the latest value', () => {
        const emitted: number[] = [];
        const throttle = createValueThrottle<number>((value) =>
            emitted.push(value)
        );

        throttle.push(1, 1000);
        vi.advanceTimersByTime(100);
        throttle.push(2, 1000);
        vi.advanceTimersByTime(100);
        throttle.push(3, 1000);
        vi.advanceTimersByTime(100);
        throttle.push(4, 1000);

        expect(emitted).toEqual([1]);

        vi.advanceTimersByTime(1000);

        expect(emitted).toEqual([1, 4]);
    });

    it('schedules at most one trailing timer per window and leads again afterwards', () => {
        const emitted: number[] = [];
        const throttle = createValueThrottle<number>((value) =>
            emitted.push(value)
        );

        throttle.push(1, 1000);
        vi.advanceTimersByTime(200);
        throttle.push(2, 1000);
        throttle.push(3, 1000);
        vi.advanceTimersByTime(800);

        expect(emitted).toEqual([1, 3]);

        vi.advanceTimersByTime(1000);
        throttle.push(4, 1000);

        expect(emitted).toEqual([1, 3, 4]);
    });

    it('cancels a pending trailing emit on dispose', () => {
        const emitted: number[] = [];
        const throttle = createValueThrottle<number>((value) =>
            emitted.push(value)
        );

        throttle.push(1, 1000);
        vi.advanceTimersByTime(100);
        throttle.push(2, 1000);
        throttle.dispose();
        vi.advanceTimersByTime(1000);

        expect(emitted).toEqual([1]);
    });
});
