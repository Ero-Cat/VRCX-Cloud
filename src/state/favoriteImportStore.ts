import { create } from 'zustand';

import type { FavoriteEntityKind } from '@/platform/tauri/bindings';

type FavoriteImportType = FavoriteEntityKind;
export type FavoriteImportRow = {
    id: string;
    [key: string]: unknown;
};
type FavoriteImportOpenOptions = {
    type?: FavoriteImportType;
    input?: string;
};
type FavoriteImportStore = {
    open: boolean;
    type: FavoriteImportType;
    input: string;
    rows: FavoriteImportRow[];
    loading: boolean;
    progress: number;
    progressTotal: number;
    importProgress: number;
    importProgressTotal: number;
    errors: string;
    remoteGroupName: string;
    localGroupName: string;
    sessionId: number;
    openDialog(options?: FavoriteImportOpenOptions): void;
    closeDialog(): void;
    cancelActiveWork(): void;
    setInput(input: string): void;
    setLoading(loading: boolean): void;
    setProgress(progress: number, progressTotal: number): void;
    setImportProgress(
        importProgress: number,
        importProgressTotal: number
    ): void;
    setErrors(errors: string): void;
    appendError(error: string): void;
    setRows(rows: FavoriteImportRow[]): void;
    addRow(row: FavoriteImportRow | null | undefined): void;
    removeRow(id: string): void;
    clearRows(): void;
    setRemoteGroupName(remoteGroupName: string): void;
    setLocalGroupName(localGroupName: string): void;
    resetImportState(): void;
};

type FavoriteImportState = Pick<
    FavoriteImportStore,
    | 'open'
    | 'type'
    | 'input'
    | 'rows'
    | 'loading'
    | 'progress'
    | 'progressTotal'
    | 'importProgress'
    | 'importProgressTotal'
    | 'errors'
    | 'remoteGroupName'
    | 'localGroupName'
    | 'sessionId'
>;

const initialState: FavoriteImportState = {
    open: false,
    type: 'avatar',
    input: '',
    rows: [],
    loading: false,
    progress: 0,
    progressTotal: 0,
    importProgress: 0,
    importProgressTotal: 0,
    errors: '',
    remoteGroupName: '',
    localGroupName: '',
    sessionId: 0
};

export const useFavoriteImportStore = create<FavoriteImportStore>((set) => ({
    ...initialState,
    openDialog({ type, input = '' }: FavoriteImportOpenOptions = {}) {
        set((state) => {
            return {
                ...initialState,
                open: true,
                type: type ?? 'avatar',
                input,
                sessionId: state.sessionId + 1
            };
        });
    },
    closeDialog() {
        set((state) => ({
            ...initialState,
            sessionId: state.sessionId + 1
        }));
    },
    cancelActiveWork() {
        set((state) => ({
            ...state,
            loading: false,
            progress: 0,
            progressTotal: 0,
            importProgress: 0,
            importProgressTotal: 0,
            sessionId: state.sessionId + 1
        }));
    },
    setInput(input) {
        set({ input });
    },
    setLoading(loading) {
        set({ loading });
    },
    setProgress(progress, progressTotal) {
        set({ progress, progressTotal });
    },
    setImportProgress(importProgress, importProgressTotal) {
        set({ importProgress, importProgressTotal });
    },
    setErrors(errors) {
        set({ errors });
    },
    appendError(error) {
        if (!error) {
            return;
        }
        set((state) => ({
            errors: `${state.errors || ''}${error}${error.endsWith('\n') ? '' : '\n'}`
        }));
    },
    setRows(rows) {
        set({ rows });
    },
    addRow(row) {
        if (!row?.id) {
            return;
        }
        set((state) => {
            if (state.rows.some((entry) => entry.id === row.id)) {
                return state;
            }
            return { rows: [...state.rows, row] };
        });
    },
    removeRow(id) {
        set((state) => ({
            rows: state.rows.filter((row) => row.id !== id)
        }));
    },
    clearRows() {
        set({ rows: [] });
    },
    setRemoteGroupName(remoteGroupName) {
        set({
            remoteGroupName,
            localGroupName: remoteGroupName ? '' : ''
        });
    },
    setLocalGroupName(localGroupName) {
        set({
            localGroupName,
            remoteGroupName: localGroupName ? '' : ''
        });
    },
    resetImportState() {
        set((state) => ({
            ...initialState,
            open: state.open,
            type: state.type
        }));
    }
}));
