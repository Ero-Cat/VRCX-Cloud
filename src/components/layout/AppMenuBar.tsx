import { type ComponentProps, type ReactNode, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';

import { AboutVrcxDialog } from '@/components/about/AboutDialog';
import { OpenSourceNoticeDialog } from '@/components/hosts/system-dialogs/OpenSourceNoticeDialog';
import { cn } from '@/lib/utils';
import { logoutFromReactShell } from '@/services/authExecutionService';
import { openExternalLink } from '@/services/entityMediaService';
import { toast } from '@/services/toastService';
import { getBuildBadgeLabel } from '@/shared/buildLabel';
import { links } from '@/shared/constants/link';
import { formatReleaseDisplayVersion } from '@/shared/utils/releaseVersion';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Badge } from '@/ui/shadcn/badge';
import {
    Menubar,
    MenubarContent,
    MenubarGroup,
    MenubarItem,
    MenubarLabel,
    MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger
} from '@/ui/shadcn/menubar';

function MenuItem({
    children,
    onClick,
    className,
    ...props
}: ComponentProps<typeof MenubarItem>) {
    return (
        <MenubarItem
            className={cn('min-h-7 min-w-48 text-xs', className)}
            onClick={onClick}
            {...props}
        >
            {children}
        </MenubarItem>
    );
}

function MenuGroupLabel({ children }: { children: ReactNode }) {
    return (
        <MenubarLabel className="text-muted-foreground px-2 py-1.5 text-[11px] font-medium uppercase">
            {children}
        </MenubarLabel>
    );
}

export function AppMenuBar() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const [aboutOpen, setAboutOpen] = useState(false);
    const [openSourceNoticeOpen, setOpenSourceNoticeOpen] = useState(false);
    const setSystemHostOpen = useRuntimeStore(
        (state) => state.setSystemHostOpen
    );
    const appVersion = formatReleaseDisplayVersion(VERSION || '') || '-';
    const buildBadgeLabel = getBuildBadgeLabel(t);

    async function runLogout() {
        try {
            await logoutFromReactShell();
        } catch (error) {
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('app_menu.messages.logout_failed')
            });
        }
    }

    function openLink(url: string) {
        openExternalLink(url);
    }

    return (
        <>
            <Menubar className="h-full border-0 bg-transparent p-0! shadow-none">
                <MenubarMenu>
                    <MenubarTrigger className="text-muted-foreground hover:text-foreground aria-expanded:text-foreground h-full rounded-none px-3 py-0! text-xs">
                        <span className="vrcx-0-brand">VRCX-Cloud</span>
                    </MenubarTrigger>
                    <MenubarContent align="start">
                        <MenubarGroup>
                            <MenuItem onClick={() => navigate('/settings')}>
                                {t('app_menu.settings')}
                            </MenuItem>
                        </MenubarGroup>
                        <MenubarSeparator />
                        <MenubarGroup>
                            <MenuItem
                                variant="destructive"
                                onClick={() => {
                                    runLogout();
                                }}
                            >
                                {t('app_menu.logout')}
                            </MenuItem>
                        </MenubarGroup>
                    </MenubarContent>
                </MenubarMenu>

                <MenubarMenu>
                    <MenubarTrigger className="text-muted-foreground hover:text-foreground aria-expanded:text-foreground h-full rounded-none px-2 !py-0 text-xs">
                        <span className="flex min-w-0 items-center gap-1.5">
                            <span>{t('app_menu.help')}</span>
                            {buildBadgeLabel ? (
                                <Badge
                                    variant="secondary"
                                    className="h-4 rounded-md px-1 text-[10px] leading-none shadow-none"
                                >
                                    {buildBadgeLabel}
                                </Badge>
                            ) : null}
                        </span>
                    </MenubarTrigger>
                    <MenubarContent align="start">
                        <MenubarGroup>
                            <MenuItem
                                onClick={() =>
                                    setSystemHostOpen('changelogOpen', true)
                                }
                            >
                                {t('nav_menu.changelog')}
                            </MenuItem>
                            <MenuItem
                                onClick={() =>
                                    setSystemHostOpen(
                                        'keyboardShortcutsOpen',
                                        true
                                    )
                                }
                            >
                                {t('app_menu.keyboard_shortcuts')}
                            </MenuItem>
                        </MenubarGroup>
                        <MenubarSeparator />
                        <MenubarGroup>
                            <MenuItem onClick={() => openLink(links.issues)}>
                                {t('app_menu.report_issue')}
                            </MenuItem>
                        </MenubarGroup>
                        <MenubarSeparator />
                        <MenubarGroup>
                            <MenuGroupLabel>
                                {t('app_menu.community')}
                            </MenuGroupLabel>
                            <MenuItem onClick={() => openLink(links.github)}>
                                GitHub
                            </MenuItem>
                        </MenubarGroup>
                        <MenubarSeparator />
                        <MenubarGroup>
                            <MenuItem
                                label={t('app_menu.about')}
                                className="min-w-56"
                                onClick={() => setAboutOpen(true)}
                            >
                                {t('app_menu.about')}
                                <MenubarShortcut className="font-mono tracking-normal tabular-nums">
                                    {appVersion}
                                </MenubarShortcut>
                            </MenuItem>
                        </MenubarGroup>
                    </MenubarContent>
                </MenubarMenu>
            </Menubar>

            <OpenSourceNoticeDialog
                open={openSourceNoticeOpen}
                onOpenChange={setOpenSourceNoticeOpen}
            />

            <AboutVrcxDialog
                open={aboutOpen}
                onOpenChange={setAboutOpen}
                onOpenLicenses={() => {
                    setAboutOpen(false);
                    setOpenSourceNoticeOpen(true);
                }}
            />
        </>
    );
}
