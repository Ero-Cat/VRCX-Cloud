// @vitest-environment jsdom

import { cleanup, render } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    restoreNormalWindowModeForIntent: vi.fn()
}));

vi.mock('@/services/windowModeService', () => ({
    restoreNormalWindowModeForIntent: mocks.restoreNormalWindowModeForIntent
}));

import { useCriticalTaskStore } from '@/state/criticalTaskStore';

import { useCriticalTask } from './useCriticalTask';

function Harness({ active }: { active: boolean }) {
    useCriticalTask('databaseUpgrade', active);
    return null;
}

beforeEach(() => {
    useCriticalTaskStore.setState({ activeTasks: [] });
    mocks.restoreNormalWindowModeForIntent.mockReset();
});

afterEach(cleanup);

describe('useCriticalTask', () => {
    it('registers the task and leaves the sidebar window when it starts', () => {
        const view = render(<Harness active={false} />);
        expect(useCriticalTaskStore.getState().activeTasks).toEqual([]);
        expect(mocks.restoreNormalWindowModeForIntent).not.toHaveBeenCalled();

        view.rerender(<Harness active />);

        expect(useCriticalTaskStore.getState().activeTasks).toEqual([
            'databaseUpgrade'
        ]);
        expect(mocks.restoreNormalWindowModeForIntent).toHaveBeenCalledOnce();
    });

    it('releases the task when it finishes', () => {
        const view = render(<Harness active />);

        view.rerender(<Harness active={false} />);

        expect(useCriticalTaskStore.getState().activeTasks).toEqual([]);
    });

    it('releases the task when the owner unmounts', () => {
        render(<Harness active />);

        cleanup();

        expect(useCriticalTaskStore.getState().activeTasks).toEqual([]);
    });
});
