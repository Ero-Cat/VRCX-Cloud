import { ArrowUpIcon, SquareIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';
import { Textarea } from '@/ui/shadcn/textarea';

interface ComposerProps {
    busy: boolean;
    disabled: boolean;
    onSend: (text: string) => void;
    onStop: () => void;
}

export function Composer({ busy, disabled, onSend, onStop }: ComposerProps) {
    const { t } = useTranslation();
    const [value, setValue] = useState('');

    const submit = () => {
        const text = value.trim();
        if (!text || disabled) {
            return;
        }
        onSend(text);
        setValue('');
    };

    return (
        <div className="border-border/40 flex items-end gap-2 border-t p-3">
            <Textarea
                value={value}
                disabled={disabled}
                onChange={(event) => setValue(event.target.value)}
                onKeyDown={(event) => {
                    if (event.key === 'Enter' && !event.shiftKey) {
                        event.preventDefault();
                        submit();
                    }
                }}
                placeholder={t('assistant.composer_placeholder')}
                className="max-h-32 min-h-10 flex-1 resize-none"
                rows={1}
            />
            {busy ? (
                <Button
                    size="icon"
                    variant="secondary"
                    onClick={onStop}
                    title={t('assistant.stop')}
                    className="size-10 shrink-0"
                >
                    <SquareIcon className="size-4" />
                </Button>
            ) : (
                <Button
                    size="icon"
                    onClick={submit}
                    disabled={disabled || !value.trim()}
                    title={t('assistant.send')}
                    className="size-10 shrink-0"
                >
                    <ArrowUpIcon className="size-4" />
                </Button>
            )}
        </div>
    );
}
