// @vitest-environment jsdom

import { act, cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { HOUR_MS, MINUTE_MS, SECOND_MS } from '@/shared/constants/time';
import { useInstanceJoinHistoryStore } from '@/state/instanceJoinHistoryStore';
import { useRuntimeStore } from '@/state/runtimeStore';

vi.mock('react-i18next', async (importOriginal) => ({
    ...(await importOriginal<typeof import('react-i18next')>()),
    useTranslation: () => ({
        t: (key: string, options?: { count?: number }) =>
            `${key}:${options?.count ?? ''}`
    })
}));

import { InstanceVisitedBadge } from './InstanceVisitedBadge';

const LOCATION = 'wrld_11111111-1111-1111-1111-111111111111:12345~region(jp)';
const OTHER_LOCATION =
    'wrld_11111111-1111-1111-1111-111111111111:67890~region(jp)';
const NOW = Date.parse('2026-07-20T12:00:00.000Z');

function setCurrentLocation(currentLocation: string) {
    useRuntimeStore.setState((state) => ({
        gameState: { ...state.gameState, currentLocation }
    }));
}

function recordJoin(joinedAtMs: number) {
    act(() => {
        useInstanceJoinHistoryStore
            .getState()
            .recordInstanceJoin(LOCATION, joinedAtMs);
    });
}

beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW);
    useInstanceJoinHistoryStore.getState().resetInstanceJoinHistory();
    setCurrentLocation('');
});

afterEach(() => {
    cleanup();
    vi.useRealTimers();
});

describe('InstanceVisitedBadge', () => {
    it('shows whole minutes under an hour and whole hours beyond', () => {
        recordJoin(NOW - 20 * SECOND_MS);
        render(<InstanceVisitedBadge location={LOCATION} />);
        expect(
            screen.getByText('side_panel.visited_minutes_ago:1')
        ).toBeTruthy();

        act(() => {
            vi.advanceTimersByTime(59 * MINUTE_MS);
        });
        expect(
            screen.getByText('side_panel.visited_minutes_ago:59')
        ).toBeTruthy();

        act(() => {
            vi.advanceTimersByTime(MINUTE_MS + 50 * MINUTE_MS);
        });
        expect(screen.getByText('side_panel.visited_hours_ago:1')).toBeTruthy();
    });

    it('restarts from the latest join when re-entered within three hours', () => {
        recordJoin(NOW - HOUR_MS);
        render(<InstanceVisitedBadge location={LOCATION} />);
        expect(screen.getByText('side_panel.visited_hours_ago:1')).toBeTruthy();

        recordJoin(NOW - MINUTE_MS);

        expect(
            screen.getByText('side_panel.visited_minutes_ago:1')
        ).toBeTruthy();
    });

    it('expires three hours after the last join and returns on re-entry', () => {
        recordJoin(NOW - 3 * HOUR_MS + MINUTE_MS);
        const { container } = render(
            <InstanceVisitedBadge location={LOCATION} />
        );
        expect(screen.getByText('side_panel.visited_hours_ago:2')).toBeTruthy();

        act(() => {
            vi.advanceTimersByTime(2 * MINUTE_MS);
        });
        expect(container.textContent).toBe('');

        recordJoin(Date.now() - 5 * MINUTE_MS);

        expect(
            screen.getByText('side_panel.visited_minutes_ago:5')
        ).toBeTruthy();
    });

    it('stays hidden for instances not joined this run', () => {
        useInstanceJoinHistoryStore
            .getState()
            .setInstanceJoinHistory([[LOCATION, NOW - HOUR_MS]]);

        const { container } = render(
            <InstanceVisitedBadge location={LOCATION} />
        );

        expect(container.textContent).toBe('');
    });

    it('hides while the user is in the instance and shows once they leave', () => {
        recordJoin(NOW - 10 * MINUTE_MS);
        setCurrentLocation(LOCATION);

        const { container } = render(
            <InstanceVisitedBadge location={LOCATION} />
        );
        expect(container.textContent).toBe('');

        act(() => setCurrentLocation(OTHER_LOCATION));

        expect(
            screen.getByText('side_panel.visited_minutes_ago:10')
        ).toBeTruthy();
    });
});
