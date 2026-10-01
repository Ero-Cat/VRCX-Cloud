import { beforeEach, describe, expect, it, vi } from 'vitest';

import type {
    TrayShortcutBinding,
    TrayShortcutError,
    TrayShortcutSnapshot,
    TrayShortcutUpdate
} from '@/platform/tauri/bindings';
import { useTrayShortcutStore } from '@/state/trayShortcutStore';

const mocks = vi.hoisted(() => ({
    get: vi.fn<() => Promise<TrayShortcutSnapshot>>(),
    set: vi.fn<
        (binding: TrayShortcutBinding | null) => Promise<TrayShortcutUpdate>
    >(),
    check: vi.fn<
        (binding: TrayShortcutBinding) => Promise<TrayShortcutError | null>
    >(),
    recording: vi.fn<(recording: boolean) => Promise<boolean>>()
}));

vi.mock('@/platform/tauri/bindings', () => ({
    commands: {
        appGetTrayShortcut: mocks.get,
        appSetTrayShortcut: mocks.set,
        appCheckTrayShortcut: mocks.check,
        appSetTrayShortcutRecording: mocks.recording
    }
}));

import {
    checkTrayShortcut,
    initializeTrayShortcut,
    setTrayShortcut,
    setTrayShortcutRecording
} from './trayShortcutService';

const binding: TrayShortcutBinding = {
    key: 'KeyV',
    control: true,
    alt: true,
    shift: false
};

beforeEach(() => {
    useTrayShortcutStore.setState({ snapshot: null });
    mocks.get.mockReset().mockResolvedValue({ binding: null, status: 'unset' });
    mocks.set.mockReset().mockImplementation(async (binding) => ({
        kind: 'saved',
        snapshot: { binding, status: binding ? 'active' : 'unset' }
    }));
    mocks.check.mockReset().mockResolvedValue(null);
    mocks.recording
        .mockReset()
        .mockImplementation(async (recording) => recording);
});

describe('tray shortcut UI synchronization', () => {
    it('shares initialization between consumers and reuses the loaded snapshot', async () => {
        await Promise.all([initializeTrayShortcut(), initializeTrayShortcut()]);
        await initializeTrayShortcut();
        expect(mocks.get).toHaveBeenCalledOnce();
    });

    it('does not overwrite a successful save with a late initialization response', async () => {
        let finishRead: (snapshot: TrayShortcutSnapshot) => void = () =>
            undefined;
        mocks.get.mockImplementationOnce(
            () =>
                new Promise((resolve) => {
                    finishRead = resolve;
                })
        );
        const initialization = initializeTrayShortcut();
        await setTrayShortcut(binding);
        finishRead({ binding: null, status: 'unset' });
        await initialization;
        expect(useTrayShortcutStore.getState().snapshot).toEqual({
            binding,
            status: 'active'
        });
    });

    it('orders recording start and stop without blocking independent checks', async () => {
        let finishStart: (active: boolean) => void = () => undefined;
        mocks.recording.mockImplementationOnce(
            () =>
                new Promise((resolve) => {
                    finishStart = resolve;
                })
        );
        const start = setTrayShortcutRecording(true);
        const stop = setTrayShortcutRecording(false);
        await Promise.resolve();
        expect(mocks.recording.mock.calls).toEqual([[true]]);
        await checkTrayShortcut(binding);
        expect(mocks.check).toHaveBeenCalledWith(binding);
        expect(mocks.recording.mock.calls).toEqual([[true]]);
        finishStart(true);
        await expect(start).resolves.toBe(true);
        await expect(stop).resolves.toBe(false);
        expect(mocks.recording.mock.calls).toEqual([[true], [false]]);
    });
});
