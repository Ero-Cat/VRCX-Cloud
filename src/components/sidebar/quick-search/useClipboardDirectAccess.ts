import { useCallback, useEffect, useEffectEvent, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { directAccessParse } from '@/services/directAccessService';
import { getClipboardText } from '@/services/shellIntegrationService';
import { toast } from '@/services/toastService';

type ClipboardCandidate = { input: string; toastId?: string };

export function useClipboardDirectAccess(
    enabled: boolean,
    session: number,
    onDirectAccess: (input: string) => void
) {
    const { t } = useTranslation();
    const candidate = useRef<ClipboardCandidate | null>(null);
    const getHintTitle = useEffectEvent(() =>
        t('side_panel.search_clipboard_detected')
    );
    const openCandidate = useEffectEvent((input: string) =>
        onDirectAccess(input)
    );

    const dismiss = useCallback(() => {
        const id = candidate.current?.toastId;
        candidate.current = null;
        if (id) {
            toast.close(id);
        }
    }, []);

    useEffect(() => {
        if (!enabled) {
            return;
        }
        const next: ClipboardCandidate = { input: '' };
        candidate.current = next;
        const handleKeyDown = (event: KeyboardEvent) => {
            const input = candidate.current?.input;
            dismiss();
            if (
                input &&
                event.key === 'Enter' &&
                !event.isComposing &&
                !event.repeat &&
                !event.ctrlKey &&
                !event.metaKey &&
                !event.altKey &&
                !event.shiftKey
            ) {
                event.preventDefault();
                event.stopPropagation();
                openCandidate(input);
            }
        };
        document.addEventListener('keydown', handleKeyDown, true);
        void getClipboardText()
            .then(async (text) => {
                const input = text.trim();
                if (!input || candidate.current !== next) {
                    return;
                }
                if (
                    !(await directAccessParse(input, 'detect')) ||
                    candidate.current !== next
                ) {
                    return;
                }
                next.toastId = toast.add({
                    type: 'info',
                    title: getHintTitle(),
                    timeout: 0,
                    onClose: () => {
                        if (candidate.current === next) {
                            candidate.current = null;
                        }
                    }
                });
                next.input = input;
            })
            .catch((error: unknown) => {
                console.warn(
                    'Clipboard direct access detection failed:',
                    error
                );
            });
        return () => {
            document.removeEventListener('keydown', handleKeyDown, true);
            dismiss();
        };
    }, [dismiss, enabled, session]);

    return dismiss;
}
