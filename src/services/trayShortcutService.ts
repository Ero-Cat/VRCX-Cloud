import {
    commands,
    type TrayShortcutBinding,
    type TrayShortcutError,
    type TrayShortcutUpdate
} from '@/platform/tauri/bindings';
import { useTrayShortcutStore } from '@/state/trayShortcutStore';

let initialization: Promise<void> | null = null;
let recordingChanges = Promise.resolve();

export function initializeTrayShortcut(): Promise<void> {
    if (useTrayShortcutStore.getState().snapshot) return Promise.resolve();
    initialization ??= commands
        .appGetTrayShortcut()
        .then((snapshot) => {
            if (!useTrayShortcutStore.getState().snapshot) {
                useTrayShortcutStore.setState({ snapshot });
            }
        })
        .finally(() => {
            initialization = null;
        });
    return initialization;
}

export async function setTrayShortcut(
    binding: TrayShortcutBinding | null
): Promise<TrayShortcutUpdate> {
    const result = await commands.appSetTrayShortcut(binding);
    useTrayShortcutStore.setState({ snapshot: result.snapshot });
    return result;
}

export function checkTrayShortcut(
    binding: TrayShortcutBinding
): Promise<TrayShortcutError | null> {
    return commands.appCheckTrayShortcut(binding);
}

export function setTrayShortcutRecording(recording: boolean): Promise<boolean> {
    const change = recordingChanges.then(() =>
        commands.appSetTrayShortcutRecording(recording)
    );
    recordingChanges = change.then(
        () => undefined,
        () => undefined
    );
    return change;
}
