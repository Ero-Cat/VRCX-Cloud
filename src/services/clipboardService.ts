import { toast } from '@/services/toastService';

export type CopyTextToClipboardOptions = {
    successMessage?: string;
    errorMessage?: string | ((error: unknown) => string);
};

export async function copyTextToClipboard(
    text: string,
    options: CopyTextToClipboardOptions = {}
): Promise<boolean> {
    try {
        await navigator.clipboard.writeText(text);
    } catch (error) {
        if (typeof options.errorMessage === 'function') {
            const message = options.errorMessage(error);
            if (message) {
                toast.add({ type: 'error', title: message });
            }
        } else if (options.errorMessage) {
            toast.add({ type: 'error', title: options.errorMessage });
        }
        return false;
    }

    if (options.successMessage) {
        toast.add({ type: 'success', title: options.successMessage });
    }
    return true;
}
