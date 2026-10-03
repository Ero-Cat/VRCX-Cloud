import { create } from 'zustand';

/**
 * Admin browser gate state for the self-hosted server build.
 *
 * When the server configures an admin password, every web command is
 * rejected with `adminAuthRequired` until the browser unlocks once at
 * `POST /api/admin/auth`. The server then sets a permanent cookie, so
 * this gate asks for the password at most once per browser.
 */
export type AdminAuthPhase =
    // Not checked yet.
    | 'unknown'
    // No gate configured (desktop build, static hosting or server
    // offline): never block the app.
    | 'disabled'
    // Gate active and this browser has not unlocked.
    | 'locked'
    // Gate active and this browser already holds a valid token.
    | 'open';

export type AdminUnlockFailure = 'wrongPassword' | 'failed';

interface AdminAuthStore {
    phase: AdminAuthPhase;
    refresh: () => Promise<void>;
    unlock: (password: string) => Promise<AdminUnlockFailure | null>;
    markLocked: () => void;
}

interface AdminStatusResponse {
    required?: boolean;
    unlocked?: boolean;
}

async function fetchAdminStatus(): Promise<AdminStatusResponse | null> {
    try {
        const response = await fetch('/api/admin/status', {
            credentials: 'same-origin'
        });
        if (!response.ok) {
            return null;
        }
        return (await response.json()) as AdminStatusResponse;
    } catch {
        return null;
    }
}

export const useAdminAuthStore = create<AdminAuthStore>((set, get) => ({
    phase: 'unknown',

    async refresh() {
        const status = await fetchAdminStatus();
        if (!status) {
            // A blocked invoke is authoritative even when the status
            // probe itself fails; otherwise never block on a missing
            // endpoint (desktop / static hosting).
            if (get().phase !== 'locked') {
                set({ phase: 'disabled' });
            }
            return;
        }
        set({
            phase: status.required
                ? status.unlocked
                    ? 'open'
                    : 'locked'
                : 'disabled'
        });
    },

    async unlock(password) {
        try {
            const response = await fetch('/api/admin/auth', {
                method: 'POST',
                credentials: 'same-origin',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ password })
            });
            if (response.ok) {
                set({ phase: 'open' });
                return null;
            }
            return response.status === 401 ? 'wrongPassword' : 'failed';
        } catch {
            return 'failed';
        }
    },

    markLocked() {
        if (get().phase !== 'locked') {
            set({ phase: 'locked' });
        }
    }
}));
