const PRINT_UPLOAD_NOTE_MAX_LENGTH = 32;

export function buildPrintUploadParams({
    note,
    timestamp
}: {
    note?: string;
    timestamp: string;
}) {
    return {
        note: (note ?? '').slice(0, PRINT_UPLOAD_NOTE_MAX_LENGTH),
        timestamp
    };
}
