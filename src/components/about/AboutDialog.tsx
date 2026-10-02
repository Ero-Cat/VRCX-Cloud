import { HeartIcon, type LucideIcon } from 'lucide-react';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { openExternalLink } from '@/services/entityMediaService';
import { links } from '@/shared/constants/link';
import { formatReleaseDisplayVersion } from '@/shared/utils/releaseVersion';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Button } from '@/ui/shadcn/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogTitle
} from '@/ui/shadcn/dialog';

const WORDMARK_FONT_LINK_ID = 'vrcx-0-about-wordmark-font';
const WORDMARK_FONT_URL =
    'https://fonts.googleapis.com/css2?family=Jost:wght@500&text=VRCX-Cloud&display=swap';

const PLATFORM_LABELS: Record<string, string> = {
    windows: 'Windows',
    macos: 'macOS',
    linux: 'Linux'
};

type AboutActionLink = {
    key: string;
    label: string;
    href: string;
    icon: LucideIcon;
};

const SUPPORT_LINKS: AboutActionLink[] = [
    {
        key: 'github-issues',
        label: 'GitHub Issues',
        href: links.issues,
        icon: HeartIcon
    }
];

function ensureWordmarkFontLoaded() {
    if (document.getElementById(WORDMARK_FONT_LINK_ID)) {
        return;
    }
    const link = document.createElement('link');
    link.id = WORDMARK_FONT_LINK_ID;
    link.rel = 'stylesheet';
    link.href = WORDMARK_FONT_URL;
    document.head.appendChild(link);
}

function getAppDisplayVersion(): string {
    return formatReleaseDisplayVersion(VERSION || '') || String(VERSION || '');
}

export function AboutVrcxDialog({
    open,
    onOpenChange,
    onOpenLicenses
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onOpenLicenses: () => void;
}) {
    const { t } = useTranslation();
    const hostPlatform = useRuntimeStore(
        (state) => state.hostCapabilities.platform
    );

    useEffect(() => {
        if (open) {
            ensureWordmarkFontLoaded();
        }
    }, [open]);

    const displayVersion = getAppDisplayVersion();
    const platformLabel = PLATFORM_LABELS[hostPlatform] || '';

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent
                showCloseButton={false}
                className="gap-0 px-7 pt-8 pb-6 sm:max-w-[560px]"
            >
                <div className="flex flex-col items-center text-center">
                    <DialogTitle
                        className="text-4xl leading-none font-medium tracking-normal select-none"
                        style={{ fontFamily: "'Jost', var(--font-sans)" }}
                    >
                        VRCX-Cloud
                    </DialogTitle>
                    <DialogDescription className="mt-3 text-[13px]">
                        {t('view.about.tagline')}
                    </DialogDescription>
                    <div className="text-muted-foreground mt-3 inline-flex h-6 items-center justify-center gap-2 font-sans text-xs tracking-[0.01em]">
                        <span className="text-foreground/80 font-medium tabular-nums">
                            {displayVersion}
                        </span>
                        {platformLabel ? (
                            <>
                                <span
                                    aria-hidden="true"
                                    className="bg-muted-foreground/35 size-1 rounded-full"
                                />
                                <span>{platformLabel}</span>
                            </>
                        ) : null}
                    </div>
                </div>

                <div className="mt-6 flex flex-col items-center gap-3">
                    <span
                        id="about-support-title"
                        className="text-muted-foreground/60 text-[10px] font-medium tracking-[0.16em] uppercase"
                    >
                        {t('support_vrcx.title')}
                    </span>
                    <div
                        className="flex flex-wrap justify-center gap-2"
                        role="group"
                        aria-labelledby="about-support-title"
                    >
                        {SUPPORT_LINKS.map(
                            ({ key, label, href, icon: Icon }) => (
                                <Button
                                    key={key}
                                    type="button"
                                    variant="outline"
                                    size="sm"
                                    className="text-muted-foreground hover:text-foreground"
                                    onClick={() => {
                                        openExternalLink(href);
                                    }}
                                >
                                    <Icon data-icon="inline-start" />
                                    {label}
                                </Button>
                            )
                        )}
                    </div>
                </div>

                <div className="mt-6 flex flex-wrap items-center justify-between gap-2 border-t pt-4">
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="text-muted-foreground/70 hover:text-foreground h-7 px-2 text-[11.5px] font-medium"
                        onClick={() => {
                            openExternalLink(links.license);
                        }}
                    >
                        {t('view.about.license_line')}
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="text-muted-foreground/70 hover:text-foreground h-7 px-2 text-[11.5px] font-medium"
                        onClick={onOpenLicenses}
                    >
                        {t('app_menu.open_source_licenses')}
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
