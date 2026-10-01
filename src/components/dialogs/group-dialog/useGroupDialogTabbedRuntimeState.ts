import { useModalStore } from '@/state/modalStore';
import { useRuntimeStore } from '@/state/runtimeStore';

export function useGroupDialogTabbedRuntimeState() {
    const currentEndpoint = useRuntimeStore(
        (state) => state.auth.currentUserEndpoint
    );
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const openImagePreview = useModalStore((state) => state.openImagePreview);
    const prompt = useModalStore((state) => state.prompt);
    const confirm = useModalStore((state) => state.confirm);

    return {
        confirm,
        currentEndpoint,
        currentUserId,
        openImagePreview,
        prompt
    };
}
