import { Button } from '@/ui/shadcn/button';

import type { useLoginPageController } from '../useLoginPageController';

type LoginActions = ReturnType<typeof useLoginPageController>['actions'];

export function LoginPageFooter({
    onOpenGithub
}: {
    onOpenGithub: LoginActions['openGithub'];
}) {
    return (
        <div className="text-muted-foreground/65 mt-4 flex shrink-0 items-center justify-center gap-x-2 text-center text-[0.7rem]">
            <Button
                type="button"
                variant="link"
                className="text-muted-foreground/75 h-auto p-0 text-[0.7rem]"
                onClick={onOpenGithub}
            >
                GitHub
            </Button>
        </div>
    );
}
