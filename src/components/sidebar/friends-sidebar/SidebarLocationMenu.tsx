import { ExternalLinkIcon, MessageSquareIcon } from 'lucide-react';
import type { ReactElement, SyntheticEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { LaunchModeContextMenuGroup } from '@/components/launch/LaunchModeContextMenuGroup';
import { isUsableInstanceLocation } from '@/components/location/locationModel';
import { selfInviteToInstance } from '@/services/launchService';
import { toast } from '@/services/toastService';
import { parseLocation } from '@/shared/utils/location';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuGroup,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger
} from '@/ui/shadcn/context-menu';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuTrigger
} from '@/ui/shadcn/dropdown-menu';

import { useSidebarMenuDoubleClick } from './useSidebarMenuDoubleClick';

export function SidebarLocationMenu({
    children,
    openOnClick,
    location,
    instanceClosed,
    onOpen
}: {
    children: ReactElement;
    openOnClick: boolean;
    location: string;
    instanceClosed: boolean;
    onOpen(event: SyntheticEvent<HTMLElement>): void;
}) {
    const { t } = useTranslation();
    const doubleClick = useSidebarMenuDoubleClick(onOpen);
    const parsedLocation = parseLocation(location);
    const canUseInstance =
        !instanceClosed && isUsableInstanceLocation(parsedLocation);

    async function selfInvite() {
        if (!canUseInstance) {
            return;
        }
        try {
            await selfInviteToInstance(
                parsedLocation.tag,
                parsedLocation.shortName
            );
            toast.add({
                type: 'success',
                title: t('message.invite.self_sent')
            });
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t(
                              'component.friends_sidebar.toast.failed_to_send_self_invite'
                          )
            });
        }
    }

    const menuItems = (
        <>
            <ContextMenuGroup>
                <ContextMenuItem onClick={onOpen}>
                    <ExternalLinkIcon />
                    {t('common.actions.view_details')}
                </ContextMenuItem>
            </ContextMenuGroup>
            <ContextMenuSeparator />
            <LaunchModeContextMenuGroup
                disabled={!canUseInstance}
                instanceClosed={instanceClosed}
                location={location}
                errorMessage={t(
                    'host.launch_dialog.toast.launch_action_failed'
                )}
            />
            <ContextMenuSeparator />
            <ContextMenuGroup>
                <ContextMenuItem
                    disabled={!canUseInstance}
                    onClick={() => void selfInvite()}
                >
                    <MessageSquareIcon />
                    {t('dialog.launch.self_invite')}
                </ContextMenuItem>
            </ContextMenuGroup>
        </>
    );

    return (
        <ContextMenu>
            <ContextMenuTrigger
                render={
                    <span className="inline-flex max-w-full min-w-0">
                        {openOnClick ? (
                            <DropdownMenu {...doubleClick.menuProps}>
                                <DropdownMenuTrigger
                                    nativeButton={false}
                                    render={children}
                                    {...doubleClick.triggerProps}
                                />
                                <DropdownMenuContent className="w-max max-w-[calc(100vw-1rem)] min-w-56">
                                    {menuItems}
                                </DropdownMenuContent>
                            </DropdownMenu>
                        ) : (
                            children
                        )}
                    </span>
                }
            />
            <ContextMenuContent className="w-max max-w-[calc(100vw-1rem)] min-w-56">
                {menuItems}
            </ContextMenuContent>
        </ContextMenu>
    );
}
