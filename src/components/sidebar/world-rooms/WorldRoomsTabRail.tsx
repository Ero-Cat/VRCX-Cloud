import { useWorldRooms } from './useWorldRooms';
import { WorldRoomsCover } from './WorldRoomsCover';

export function WorldRoomsTabRail({ worldId }: { worldId: string }) {
    const { worldQuery, rooms } = useWorldRooms(worldId);

    return (
        <>
            <WorldRoomsCover worldId={worldId} className="size-4.5" />
            {worldQuery.isPending ? null : (
                <span className="text-[10px] leading-none tabular-nums">
                    {rooms.length}
                </span>
            )}
        </>
    );
}
