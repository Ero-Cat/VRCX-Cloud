import type { TrayShortcutBinding } from '@/platform/tauri/bindings';

export function trayShortcutKeys(binding: TrayShortcutBinding): string[] {
    const keys: string[] = [];
    if (binding.control) keys.push('Ctrl');
    if (binding.alt) keys.push('Alt');
    if (binding.shift) keys.push('Shift');
    keys.push(binding.key.replace(/^(Key|Digit)/, ''));
    return keys;
}
