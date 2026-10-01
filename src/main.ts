import { createElement, StrictMode, useState } from 'react';
import { createRoot } from 'react-dom/client';

import '@/styles/globals.css';
import { installDevPerformanceTimelineGuard } from '@/app/devPerformanceTimelineGuard';
import {
    isWebPlatform,
    webAuthStatus,
    webLogin
} from '@/platform/tauri/webTransport';
import { installErrorLogging } from '@/services/errorLogService';

// only use in dev to prevent OOM from React dev tools User Timing measures
installDevPerformanceTimelineGuard();
installErrorLogging();

function WebLoginGate(): React.ReactElement {
    const [password, setPassword] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);

    const submit = async (event: React.FormEvent) => {
        event.preventDefault();
        if (busy) return;
        setBusy(true);
        setError(null);
        const ok = await webLogin(password);
        if (ok) {
            window.location.reload();
            return;
        }
        setBusy(false);
        setError('Incorrect password.');
    };

    return createElement(
        'div',
        {
            style: {
                minHeight: '100vh',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                background: 'var(--background, #0b0b0f)',
                color: 'var(--foreground, #eee)'
            }
        },
        createElement(
            'form',
            {
                onSubmit: submit,
                style: {
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '12px',
                    width: '280px',
                    padding: '32px',
                    borderRadius: '12px',
                    background: 'rgba(255,255,255,0.04)',
                    border: '1px solid rgba(255,255,255,0.1)'
                }
            },
            createElement(
                'h1',
                { style: { fontSize: '18px', margin: 0, textAlign: 'center' } },
                'VRCX-Cloud'
            ),
            createElement('input', {
                type: 'password',
                placeholder: 'Password',
                value: password,
                autofocus: true,
                onChange: (event: React.ChangeEvent<HTMLInputElement>) =>
                    setPassword(event.target.value),
                style: {
                    padding: '10px 12px',
                    borderRadius: '8px',
                    border: '1px solid rgba(255,255,255,0.16)',
                    background: 'rgba(0,0,0,0.3)',
                    color: 'inherit'
                }
            }),
            error
                ? createElement(
                      'div',
                      { style: { color: '#f87171', fontSize: '13px' } },
                      error
                  )
                : null,
            createElement(
                'button',
                {
                    type: 'submit',
                    disabled: busy,
                    style: {
                        padding: '10px 12px',
                        borderRadius: '8px',
                        border: 'none',
                        background: '#4f46e5',
                        color: 'white',
                        cursor: busy ? 'wait' : 'pointer'
                    }
                },
                busy ? 'Signing in…' : 'Sign in'
            )
        )
    );
}

async function bootstrap() {
    const rootElement = document.getElementById('root');
    if (!rootElement) {
        throw new Error('Missing #root mount node');
    }

    // Web builds gate the whole app behind the server's single password.
    if (isWebPlatform()) {
        try {
            const status = await webAuthStatus();
            if (status.authEnabled && !status.sessionValid) {
                createRoot(rootElement).render(
                    createElement(StrictMode, null, createElement(WebLoginGate))
                );
                return;
            }
        } catch {
            // Server unreachable: fall through and let the app surface
            // connection errors through its normal handling.
        }
    }

    const [, { App }] = await Promise.all([
        import('@/services/i18nService'),
        import('./app/App')
    ]);

    createRoot(rootElement).render(
        createElement(StrictMode, null, createElement(App))
    );
}

bootstrap().catch((error: unknown) => {
    console.error(error);
});
