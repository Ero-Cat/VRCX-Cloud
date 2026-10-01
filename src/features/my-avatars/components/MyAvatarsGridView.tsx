import type { Ref } from 'react';

import type {
    MyAvatarActionHandler,
    MyAvatarsGridDensityConfig,
    MyAvatarsGridRow
} from '../myAvatarsTypes';
import { MyAvatarGridCard } from './MyAvatarsViewParts';

type MyAvatarsGridViewProps = {
    densityConfig: MyAvatarsGridDensityConfig;
    gridScrollRef: Ref<HTMLDivElement>;
    gridTotalHeight: number;
    visibleGridRows: MyAvatarsGridRow[];
    gridGap: number;
    gridColumnCount: number;
    gridMinWidth: number;
    gridPadding: number;
    savingTagsAvatarId: string;
    updatingAvatarId: string;
    uploadingImageAvatarId: string;
    onAvatarAction: MyAvatarActionHandler;
};

export function MyAvatarsGridView({
    densityConfig,
    gridScrollRef,
    gridTotalHeight,
    visibleGridRows,
    gridGap,
    gridColumnCount,
    gridMinWidth,
    gridPadding,
    savingTagsAvatarId,
    updatingAvatarId,
    uploadingImageAvatarId,
    onAvatarAction
}: MyAvatarsGridViewProps) {
    return (
        <div
            ref={gridScrollRef}
            className="min-h-0 min-w-0 flex-1 overflow-auto pr-2"
        >
            <div
                className="relative min-w-0"
                style={{
                    height: `${gridTotalHeight}px`
                }}
            >
                {visibleGridRows.map((row) => (
                    <div
                        key={row.key}
                        className="absolute right-0 left-0 grid min-w-0"
                        style={{
                            height: `${row.cellHeight}px`,
                            gap: `${gridGap}px`,
                            gridTemplateColumns: `repeat(${gridColumnCount}, minmax(${gridMinWidth}px, 1fr))`,
                            transform: `translateY(${row.top}px)`
                        }}
                    >
                        {row.avatars.map((avatar) => (
                            <div
                                key={avatar.id}
                                className="min-h-0 min-w-0"
                                style={{ padding: `${gridPadding}px` }}
                            >
                                <MyAvatarGridCard
                                    avatar={avatar}
                                    densityConfig={densityConfig}
                                    isUpdating={
                                        savingTagsAvatarId === avatar.id ||
                                        updatingAvatarId === avatar.id ||
                                        uploadingImageAvatarId === avatar.id
                                    }
                                    onAction={(action, nextAvatar) => {
                                        onAvatarAction(action, nextAvatar);
                                    }}
                                />
                            </div>
                        ))}
                    </div>
                ))}
            </div>
        </div>
    );
}
