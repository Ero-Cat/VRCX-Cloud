// @vitest-environment jsdom

import {
    act,
    cleanup,
    fireEvent,
    render,
    screen
} from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { toast } from '@/services/toastService';

import { AppToaster } from './AppToaster';

vi.mock('@/services/i18nService', () => ({
    default: {
        t: (key: string) => (key === 'common.actions.close' ? 'Close' : key)
    }
}));

afterEach(cleanup);

describe('application toaster', () => {
    it('renders a permanent login error with a working close button', async () => {
        render(<AppToaster />);
        const onClose = vi.fn();
        await act(async () => {
            toast.add({
                title: 'Login failed',
                type: 'error',
                timeout: 0,
                data: { closeButton: true },
                onClose
            });
        });
        expect(screen.getByText('Login failed')).toBeTruthy();
        const viewport = screen
            .getByText('Login failed')
            .closest('[data-slot="toast-viewport"]');
        if (!viewport) throw new Error('Missing toast viewport');
        fireEvent.mouseEnter(viewport);
        fireEvent.click(screen.getByRole('button', { name: 'Close' }));
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it('keeps positions separate and updates a progress toast without replaying its animation', async () => {
        render(<AppToaster />);
        await act(async () => {
            toast.add({ title: 'Update ready', position: 'bottom-right' });
            toast.add({ title: 'Activity hint', position: 'bottom-center' });
            toast.add({ title: 'Export 1', type: 'loading', id: 'export' });
            toast.add({ title: 'Export 2', type: 'loading', id: 'export' });
        });
        expect(
            screen
                .getByText('Update ready')
                .closest('[data-slot="toast-viewport"]')
                ?.getAttribute('data-position')
        ).toBe('bottom-right');
        expect(
            screen
                .getByText('Activity hint')
                .closest('[data-slot="toast-viewport"]')
                ?.getAttribute('data-position')
        ).toBe('bottom-center');
        expect(screen.queryByText('Export 1')).toBeNull();
        const progress = screen
            .getByText('Export 2')
            .closest('[data-type="loading"]');
        expect(progress).toBeTruthy();
        expect(progress?.className).not.toContain('animate-toast-success');
    });

    it('executes the cancel action and closes the notification once', async () => {
        render(<AppToaster />);
        const onCancel = vi.fn();
        const onClose = vi.fn();
        await act(async () => {
            toast.add({
                title: 'Exporting',
                type: 'loading',
                onClose,
                actionProps: { children: 'Cancel', onClick: onCancel }
            });
        });
        fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
        expect(onCancel).toHaveBeenCalledTimes(1);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it('allows an action to keep the notification open', async () => {
        render(<AppToaster />);
        const onClose = vi.fn();
        await act(async () => {
            toast.add({
                title: 'Review',
                timeout: 0,
                onClose,
                actionProps: {
                    children: 'Inspect',
                    onClick: (event) => event.preventDefault()
                }
            });
        });
        fireEvent.click(screen.getByRole('button', { name: 'Inspect' }));
        expect(onClose).not.toHaveBeenCalled();
    });

    it('preserves custom icons and explicit icon suppression', async () => {
        render(<AppToaster />);
        await act(async () => {
            toast.add({
                title: 'Custom',
                data: { icon: <span>Custom icon</span> }
            });
            toast.add({ title: 'No icon', type: 'info', data: { icon: null } });
        });
        expect(screen.getByText('Custom icon')).toBeTruthy();
        expect(
            screen
                .getByText('No icon')
                .closest('[data-type="info"]')
                ?.querySelector('[data-slot="toast-icon"]')
        ).toBeNull();
    });
});
