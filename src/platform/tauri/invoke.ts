import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

export async function invokeTauri<TReturn = unknown>(
    command: string,
    args?: InvokeArgs
): Promise<TReturn> {
    return invoke<TReturn>(command, args);
}
