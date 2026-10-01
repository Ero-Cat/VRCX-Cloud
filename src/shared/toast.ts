import type { ReactNode } from 'react';

export type AppToastData = {
    closeButton?: boolean;
    icon?: ReactNode;
};

export type AppToastRenderData = AppToastData & {
    closeLabel: string;
};
