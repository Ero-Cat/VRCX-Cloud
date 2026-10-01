// @vitest-environment jsdom

import { cleanup, render } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { useCriticalTaskStore } from '@/state/criticalTaskStore';

import { useCriticalTask } from './useCriticalTask';

function Harness({ active }: { active: boolean }) {
    useCriticalTask('databaseUpgrade', active);
    return null;
}

beforeEach(() => {
    useCriticalTaskStore.setState({ activeTasks: [] });
});

afterEach(cleanup);

describe('useCriticalTask', () => {
    it('registers the task when it starts', () => {
        const view = render(<Harness active={false} />);
        expect(useCriticalTaskStore.getState().activeTasks).toEqual([]);

        view.rerender(<Harness active />);

        expect(useCriticalTaskStore.getState().activeTasks).toEqual([
            'databaseUpgrade'
        ]);
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
