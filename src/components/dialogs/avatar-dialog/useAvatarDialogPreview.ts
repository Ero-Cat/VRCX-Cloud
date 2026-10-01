import { useModalStore } from '@/state/modalStore';

export function useAvatarDialogPreview() {
    return useModalStore((state) => state.openImagePreview);
}
