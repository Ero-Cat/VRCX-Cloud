import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { UserStatus } from '@/platform/tauri/bindings';
import configRepository from '@/repositories/configRepository';
import { toast } from '@/services/toastService';

import {
    maxStatusPresets,
    normalizeSelfStatusInput,
    normalizeSocialStatusPreset,
    statusPresetsConfigKey,
    type SocialStatusPreset
} from './userProfileFields';

export type { SocialStatusPreset } from './userProfileFields';

export type SocialStatusDraft = {
    status: UserStatus;
    statusDescription: string;
};

export function useSelfStatusPresets({
    socialStatusDraft
}: {
    socialStatusDraft: SocialStatusDraft;
}) {
    const { t } = useTranslation();
    const [statusPresets, setStatusPresets] = useState<SocialStatusPreset[]>(
        []
    );

    useEffect(() => {
        let active = true;

        configRepository
            .getArray<unknown>(statusPresetsConfigKey, [])
            .then((presets) => {
                if (active) {
                    setStatusPresets(
                        (presets ?? []).map(normalizeSocialStatusPreset)
                    );
                }
            })
            .catch(() => {
                if (active) {
                    setStatusPresets([]);
                }
            });

        return () => {
            active = false;
        };
    }, []);

    async function saveSelfStatusPreset() {
        const nextStatus = normalizeSelfStatusInput(socialStatusDraft.status);
        if (!nextStatus) {
            toast.add({
                type: 'warning',
                title: t(
                    'dialog.user.label.please_choose_a_valid_social_status'
                )
            });
            return;
        }

        const nextPreset: SocialStatusDraft = {
            status: nextStatus,
            statusDescription: String(
                socialStatusDraft.statusDescription || ''
            ).slice(0, 32)
        };
        if (
            statusPresets.some(
                (preset) =>
                    preset.status === nextPreset.status &&
                    (preset.statusDescription ?? '') ===
                        nextPreset.statusDescription
            )
        ) {
            toast.add({
                type: 'info',
                title: t('dialog.user.label.status_preset_already_exists')
            });
            return;
        }
        if (statusPresets.length >= maxStatusPresets) {
            toast.add({
                type: 'warning',
                title: t(
                    'dialog.user.dynamic.status_presets_are_limited_to_value',
                    {
                        value: maxStatusPresets
                    }
                )
            });
            return;
        }

        const previousPresets = statusPresets;
        const nextPresets = [...previousPresets, nextPreset];
        setStatusPresets(nextPresets);
        try {
            await configRepository.setArray(
                statusPresetsConfigKey,
                nextPresets
            );
            toast.add({
                type: 'success',
                title: t('dialog.user.success.status_preset_saved')
            });
        } catch (error) {
            setStatusPresets(previousPresets);
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('dialog.user.toast.failed_to_save_status_preset')
            });
        }
    }

    async function removeSelfStatusPreset(index: number) {
        const previousPresets = statusPresets;
        const nextPresets = previousPresets.filter(
            (_, presetIndex) => presetIndex !== index
        );
        setStatusPresets(nextPresets);
        try {
            await configRepository.setArray(
                statusPresetsConfigKey,
                nextPresets
            );
        } catch (error) {
            setStatusPresets(previousPresets);
            toast.add({
                type: 'error',
                title:
                    error instanceof Error
                        ? error.message
                        : t('dialog.user.toast.failed_to_remove_status_preset')
            });
        }
    }

    return {
        onRemovePreset: removeSelfStatusPreset,
        onSavePreset: saveSelfStatusPreset,
        statusPresets
    };
}
