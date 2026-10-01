import { ChevronDownIcon } from 'lucide-react';
import { useId, useState } from 'react';
import type { ReactNode } from 'react';

import { cn } from '@/lib/utils';

export function Exhibit({
    label,
    headline,
    unit,
    caption,
    aside,
    detailLabel,
    detail,
    footer,
    children,
    className
}: {
    label: string;
    headline?: ReactNode;
    unit?: string;
    caption?: ReactNode;
    aside?: ReactNode;
    detailLabel?: string;
    detail?: ReactNode;
    footer?: ReactNode;
    children?: ReactNode;
    className?: string;
}) {
    const [open, setOpen] = useState(false);
    const detailId = useId();

    return (
        <section
            className={cn('activity-card overflow-visible p-6', className)}
        >
            <div className="flex flex-wrap items-end justify-between gap-x-8 gap-y-3">
                <div className="min-w-0">
                    <p className="text-foreground/85 text-sm font-medium">
                        {label}
                    </p>
                    {headline !== undefined ? (
                        <p className="mt-1.5 flex items-baseline gap-1.5">
                            <span className="text-foreground text-[2.75rem] leading-[1.05] font-semibold tracking-[-0.025em] tabular-nums">
                                {headline}
                            </span>
                            {unit ? (
                                <span className="text-muted-foreground text-base">
                                    {unit}
                                </span>
                            ) : null}
                        </p>
                    ) : null}
                    {caption ? (
                        <p className="text-muted-foreground mt-1.5 text-xs tabular-nums">
                            {caption}
                        </p>
                    ) : null}
                </div>
                {aside}
            </div>

            {children ? <div className="mt-5">{children}</div> : null}

            {detail ? (
                <>
                    <button
                        type="button"
                        aria-expanded={open}
                        aria-controls={detailId}
                        onClick={() => setOpen((value) => !value)}
                        className="text-muted-foreground hover:text-foreground mt-5 flex w-full items-center justify-between gap-2 border-t border-[var(--act-edge)] pt-3 text-xs transition-colors"
                    >
                        <span>{detailLabel}</span>
                        <ChevronDownIcon
                            className={cn(
                                'size-4 text-[var(--act-accent)] transition-transform duration-300 ease-out',
                                open ? 'rotate-180' : ''
                            )}
                        />
                    </button>
                    <div
                        id={detailId}
                        className="activity-fold"
                        data-open={open}
                    >
                        <div>
                            <div className="pt-4">{detail}</div>
                        </div>
                    </div>
                </>
            ) : null}
            {footer}
        </section>
    );
}
