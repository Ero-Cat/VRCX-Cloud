import { AppToaster } from '@/components/hosts/AppToaster';
import { DialogHost } from '@/components/hosts/DialogHost';
import { FriendProfileLoadHost } from '@/components/hosts/FriendProfileLoadHost';
import { LaunchDialogHost } from '@/components/hosts/LaunchDialogHost';
import { ModalHost } from '@/components/hosts/ModalHost';
import { NotificationHost } from '@/components/hosts/NotificationHost';
import { PreviousInstancesDialogHost } from '@/components/hosts/PreviousInstancesDialogHost';
import { SystemDialogsHost } from '@/components/hosts/SystemDialogsHost';
import { ToolsDialogsHost } from '@/components/hosts/ToolsDialogsHost';
import { AssistantDialogHost } from '@/features/assistant/AssistantDialogHost';
import { VrcNotificationCenterHost } from '@/features/notifications/VrcNotificationCenterHost';
import { PrivacyLockDialogHost } from '@/features/privacy-lock/PrivacyLockDialogHost';
import { PrivacyLockOverlay } from '@/features/privacy-lock/PrivacyLockOverlay';

export function GlobalHosts() {
    return (
        <>
            <AppToaster />
            <ModalHost />
            <DialogHost />
            <FriendProfileLoadHost />
            <NotificationHost />
            <VrcNotificationCenterHost />
            <LaunchDialogHost />
            <PreviousInstancesDialogHost />
            <SystemDialogsHost />
            <ToolsDialogsHost />
            <AssistantDialogHost />
            <PrivacyLockDialogHost />
            <PrivacyLockOverlay />
        </>
    );
}
