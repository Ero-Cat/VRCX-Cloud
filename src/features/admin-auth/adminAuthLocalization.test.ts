// @vitest-environment jsdom

import { describe, expect, it } from 'vitest';

import i18nService, { setI18nLanguage } from '@/services/i18nService';

describe('admin auth dialog localization', () => {
    it('renders the unlock dialog copy in Simplified Chinese', async () => {
        await setI18nLanguage('zh-CN');

        expect(i18nService.t('admin_auth.title')).toBe('管理员验证');
        expect(i18nService.t('admin_auth.field.password')).toBe('管理员密码');
        expect(i18nService.t('admin_auth.action.unlock')).toBe('解锁');
        expect(i18nService.t('admin_auth.error.wrong_password')).toBe(
            '管理员密码不正确'
        );
        expect(i18nService.t('admin_auth.error.failed')).toBe(
            '验证失败，请重试'
        );
    });

    it('keeps the full dialog copy in the fallback locale', () => {
        expect(i18nService.exists('admin_auth.description')).toBe(true);
        expect(i18nService.exists('admin_auth.title')).toBe(true);
    });
});
