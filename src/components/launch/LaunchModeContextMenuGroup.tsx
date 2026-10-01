import { Gamepad2Icon, MonitorIcon, RectangleGogglesIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { isUsableInstanceLocation } from '@/components/location/locationModel';
import { tryOpenLaunchLocation } from '@/services/directAccessService';
import { launchVrchat } from '@/services/launchService';
import { toast } from '@/services/toastService';
import { parseLocation } from '@/shared/utils/location';
import { ContextMenuGroup, ContextMenuItem } from '@/ui/shadcn/context-menu';

export function LaunchModeContextMenuGroup({
    disabled,
    errorMessage,
    location,
    instanceClosed = false,
    shortName = ''
}: {
    disabled: boolean;
    errorMessage: string;
    location: string;
    instanceClosed?: boolean;
    shortName?: string;
}) {
    const { t } = useTranslation();
    const canOpenInGame =
        !instanceClosed && isUsableInstanceLocation(parseLocation(location));

    async function openInGame() {
        if (!canOpenInGame) {
            return;
        }
        try {
            const opened = await tryOpenLaunchLocation(location, shortName);
            if (opened) {
                toast.add({
                    type: 'success',
                    title: t(
                        'dialog.instance.success.vrchat_launch_request_sent'
                    )
                });
            } else {
                toast.add({
                    type: 'error',
                    title: t(
                        'dialog.instance.error.unable_to_open_this_instance_in_vrchat'
                    )
                });
            }
        } catch (error) {
            toast.add({
                type: 'error',
                title: error instanceof Error ? error.message : errorMessage
            });
        }
    }

    async function launch(desktopMode: boolean) {
        try {
            await launchVrchat(location, shortName, desktopMode);
        } catch (error) {
            toast.add({
                type: 'error',
                title: error instanceof Error ? error.message : errorMessage
            });
        }
    }

    return (
        <ContextMenuGroup className="grid grid-cols-2 gap-0.5">
            <ContextMenuItem
                className="col-span-2 justify-center"
                disabled={!canOpenInGame}
                onClick={() => {
                    void openInGame();
                }}
            >
                <Gamepad2Icon />
                {t('dialog.instance.action.open_in_game')}
            </ContextMenuItem>
            <ContextMenuItem
                className="justify-center"
                disabled={disabled}
                onClick={() => {
                    void launch(false);
                }}
            >
                <RectangleGogglesIcon />
                {t('dialog.launch.tile.vr')}
            </ContextMenuItem>
            <ContextMenuItem
                className="justify-center"
                disabled={disabled}
                onClick={() => {
                    void launch(true);
                }}
            >
                <MonitorIcon />
                {t('dialog.launch.tile.desktop')}
            </ContextMenuItem>
        </ContextMenuGroup>
    );
}
