import type { ReactNode } from 'react';

import { cn } from '@/lib/utils';
import { Button } from '@/ui/shadcn/button';

export function RankRow({
    index,
    onClick,
    leading,
    title,
    primary,
    secondary
}: {
    index: number;
    onClick: () => void;
    leading?: ReactNode;
    title: ReactNode;
    primary: ReactNode;
    secondary?: ReactNode;
}) {
    return (
        <Button
            type="button"
            variant="ghost"
            className="h-auto w-full items-center justify-start gap-3 rounded-none px-2 py-2 text-left font-normal transition-colors duration-100 ease-out hover:bg-[var(--act-track)]"
            onClick={onClick}
        >
            <span
                className={cn(
                    'w-4 shrink-0 text-center text-sm tabular-nums',
                    index === 0
                        ? 'text-foreground font-semibold'
                        : 'text-muted-foreground font-medium'
                )}
            >
                {index + 1}
            </span>
            {leading}
            <span className="min-w-0 flex-1">
                <span className="text-foreground block truncate text-sm font-medium">
                    {title}
                </span>
                {secondary ? (
                    <span className="text-muted-foreground block truncate text-xs">
                        {secondary}
                    </span>
                ) : null}
            </span>
            <span className="text-foreground shrink-0 text-sm tabular-nums">
                {primary}
            </span>
        </Button>
    );
}
