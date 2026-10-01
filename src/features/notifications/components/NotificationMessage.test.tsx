// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { NotificationRow } from '@/repositories/notificationPersistenceRepository';
import { openUserDialog } from '@/services/dialogService';

import { NotificationMessage } from './NotificationMessage';

vi.mock('@/services/dialogService', () => ({ openUserDialog: vi.fn() }));

afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

const userId = 'usr_00000000-0000-0000-0000-000000000001';
const name = 'Maple [猫]';
const message = `${name} has invited you to Maple Club!`;
const invitation: NotificationRow = {
    type: 'group.invite',
    senderUserId: userId,
    data: { managerUserDisplayName: name, groupName: 'Maple Club' }
};

describe('NotificationMessage', () => {
    it('opens the inviter by the notification user ID and preserves the message', () => {
        const { container } = render(
            <NotificationMessage notification={invitation} message={message} />
        );
        expect(container.textContent).toBe(message);
        expect(openUserDialog).not.toHaveBeenCalled();
        fireEvent.click(screen.getByRole('button', { name }));
        expect(openUserDialog).toHaveBeenCalledExactlyOnceWith({
            userId,
            title: name
        });
    });

    it('uses the sender username when the notification has no manager display name', () => {
        render(
            <NotificationMessage
                notification={{ ...invitation, data: {}, senderUsername: name }}
                message={message}
            />
        );
        fireEvent.click(screen.getByRole('button', { name }));
        expect(openUserDialog).toHaveBeenCalledExactlyOnceWith({
            userId,
            title: name
        });
    });

    it.each<NotificationRow>([
        { ...invitation, senderUserId: undefined },
        { ...invitation, senderUserId: '' },
        {
            ...invitation,
            senderUserId: 'grp_00000000-0000-0000-0000-000000000001'
        },
        { ...invitation, senderUserId: 'usr_invalid' },
        { ...invitation, data: {} },
        { ...invitation, data: { managerUserDisplayName: 42 } },
        { ...invitation, data: { managerUserDisplayName: ' ' } },
        { ...invitation, data: { managerUserDisplayName: 'Someone else' } },
        { ...invitation, type: 'group.announcement' }
    ])(
        'keeps unsupported or incomplete notifications as plain text: %j',
        (notification) => {
            const { container } = render(
                <NotificationMessage
                    notification={notification}
                    message={message}
                />
            );
            expect(container.textContent).toBe(message);
            expect(screen.queryByRole('button')).toBeNull();
            expect(openUserDialog).not.toHaveBeenCalled();
        }
    );
});
