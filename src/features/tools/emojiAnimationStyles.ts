import type { ImageAnimationStyle } from '@/platform/tauri/bindings';
import type { EmojiAnimationStyleName } from '@/shared/constants/emoji';

export const emojiAnimationStyleValues: Record<
    EmojiAnimationStyleName,
    ImageAnimationStyle
> = {
    Aura: 'aura',
    Bats: 'bats',
    Bees: 'bees',
    Bounce: 'bounce',
    Cloud: 'cloud',
    Confetti: 'confetti',
    Crying: 'crying',
    Dislike: 'dislike',
    Fire: 'fire',
    Idea: 'idea',
    Lasers: 'lasers',
    Like: 'like',
    Magnet: 'magnet',
    Mistletoe: 'mistletoe',
    Money: 'money',
    Noise: 'noise',
    Orbit: 'orbit',
    Pizza: 'pizza',
    Rain: 'rain',
    Rotate: 'rotate',
    Shake: 'shake',
    Snow: 'snow',
    Snowball: 'snowball',
    Spin: 'spin',
    Splash: 'splash',
    Stop: 'stop',
    ZZZ: 'zzz'
};
