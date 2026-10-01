import { cn } from '@/lib/utils';
import { getAvailablePlatforms } from '@/shared/utils/avatarPlatform';

const DOT_CLASS =
    'size-2.5 -ml-1 rounded-full border border-background/80 opacity-80 shadow-sm first:ml-0';

export function AvatarPlatformDots({
    unityPackages,
    className
}: {
    unityPackages: unknown;
    className?: string;
}) {
    const platforms = getAvailablePlatforms(unityPackages);
    if (!platforms.isQuest && !platforms.isIos) {
        return null;
    }
    return (
        <div className={cn('flex', className)}>
            {platforms.isPC ? (
                <span className={cn(DOT_CLASS, 'bg-platform-pc')} />
            ) : null}
            {platforms.isQuest ? (
                <span className={cn(DOT_CLASS, 'bg-platform-quest')} />
            ) : null}
            {platforms.isIos ? (
                <span className={cn(DOT_CLASS, 'bg-platform-ios')} />
            ) : null}
        </div>
    );
}
