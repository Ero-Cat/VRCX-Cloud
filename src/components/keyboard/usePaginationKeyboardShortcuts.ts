import { useEffect, useEffectEvent, useRef } from 'react';

const KEYBOARD_CONTROL_SELECTOR = [
    'input',
    'textarea',
    'select',
    '[contenteditable]:not([contenteditable="false"])',
    '[role="textbox"]',
    '[role="combobox"]',
    '[role="listbox"]',
    '[role="menu"]',
    '[role="menubar"]',
    '[role="tablist"]',
    '[role="slider"]',
    '[role="spinbutton"]',
    '[role="tree"]',
    '[role="grid"]'
].join(',');
const DIALOG_SELECTOR = '[role="dialog"], [role="alertdialog"]';
const POPUP_SELECTOR =
    '[role="menu"], [role="listbox"], [data-slot="popover-content"]';

export function usePaginationKeyboardShortcuts<TElement extends HTMLElement>({
    enabled = true,
    canPrevious,
    canNext,
    onPrevious,
    onNext
}: {
    enabled?: boolean;
    canPrevious: boolean;
    canNext: boolean;
    onPrevious(): void;
    onNext(): void;
}) {
    const paginationRef = useRef<TElement>(null);
    const handleKeyDown = useEffectEvent((event: KeyboardEvent) => {
        if (
            event.defaultPrevented ||
            event.repeat ||
            event.isComposing ||
            event.altKey ||
            event.ctrlKey ||
            event.metaKey ||
            event.shiftKey ||
            (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight')
        ) {
            return;
        }

        const pagination = paginationRef.current;
        if (!pagination || pagination.getClientRects().length === 0) {
            return;
        }
        const target = event.target instanceof Element ? event.target : null;
        const focused = document.activeElement;
        if (
            target?.closest(KEYBOARD_CONTROL_SELECTOR) ||
            focused?.closest(KEYBOARD_CONTROL_SELECTOR) ||
            window.getSelection()?.isCollapsed === false
        ) {
            return;
        }

        for (const dialog of document.querySelectorAll(DIALOG_SELECTOR)) {
            if (
                dialog.getClientRects().length &&
                !dialog.contains(pagination)
            ) {
                return;
            }
        }
        for (const popup of document.querySelectorAll(POPUP_SELECTOR)) {
            if (popup.getClientRects().length) {
                return;
            }
        }

        const scope =
            pagination.closest(
                `[role="tabpanel"], .vrcx-0-page-scaffold, ${DIALOG_SELECTOR}`
            ) ?? pagination.parentElement;
        if (focused && focused !== document.body && !scope?.contains(focused)) {
            return;
        }

        if (event.key === 'ArrowLeft' ? canPrevious : canNext) {
            event.preventDefault();
            if (event.key === 'ArrowLeft') {
                onPrevious();
            } else {
                onNext();
            }
        }
    });

    useEffect(() => {
        if (!enabled) {
            return;
        }
        const listener = (event: KeyboardEvent) => handleKeyDown(event);
        window.addEventListener('keydown', listener);
        return () => window.removeEventListener('keydown', listener);
    }, [enabled]);

    return paginationRef;
}
