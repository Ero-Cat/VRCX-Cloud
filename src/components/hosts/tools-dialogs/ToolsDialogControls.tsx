import { Textarea } from '@/ui/shadcn/textarea';

export function ToolTextarea({
    value,
    rows = 15
}: {
    value: string;
    rows?: number;
}) {
    return (
        <Textarea
            readOnly
            rows={rows}
            value={value}
            className="font-mono text-xs"
            onClick={(event) => event.currentTarget.select()}
        />
    );
}
