import type { Menu } from '@base-ui/react/menu';
import { useRef, type MouseEvent } from 'react';

export function useSidebarMenuDoubleClick(
    onOpen: (event: MouseEvent<HTMLElement>) => void
) {
    const actionsRef = useRef<Menu.Root.Actions | null>(null);
    const doubleClicked = useRef(false);

    function resetDoubleClick() {
        doubleClicked.current = false;
    }

    return {
        menuProps: {
            actionsRef,
            modal: false,
            onOpenChange(open: boolean, details: Menu.Root.ChangeEventDetails) {
                // Base UI can finish processing the first press after dblclick.
                if (open && doubleClicked.current) {
                    details.cancel();
                }
            }
        },
        triggerProps: {
            onPointerDown: resetDoubleClick,
            onKeyDown: resetDoubleClick,
            onDoubleClick(event: MouseEvent<HTMLElement>) {
                event.stopPropagation();
                doubleClicked.current = true;
                actionsRef.current?.close();
                onOpen(event);
            }
        }
    };
}
