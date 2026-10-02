import { normalizePlatformError } from './errors';

export type WindowResizeDirection =
    | 'North'
    | 'NorthEast'
    | 'East'
    | 'SouthEast'
    | 'South'
    | 'SouthWest'
    | 'West'
    | 'NorthWest';

export type WindowTheme = 'light' | 'dark';

export type WindowPhysicalPosition = {
    x: number;
    y: number;
};

export type WindowPhysicalSize = {
    width: number;
    height: number;
};

export type WindowSizeConstraints = {
    minWidth?: number;
    minHeight?: number;
    maxWidth?: number;
    maxHeight?: number;
};

export type WindowGeometry = {
    size: WindowPhysicalSize;
    position: WindowPhysicalPosition;
    maximized: boolean;
};

export type WindowBounds = {
    size: WindowPhysicalSize;
    position: WindowPhysicalPosition;
};

type WebviewWindowLike = {
    setZoom?: (zoom: number) => Promise<void>;
    scaleFactor?: (() => Promise<number> | number) | number;
};

type WindowLike = {
    startDragging?: () => Promise<void>;
    startResizeDragging?: (direction: WindowResizeDirection) => Promise<void>;
    minimize?: () => Promise<void>;
    maximize?: () => Promise<void>;
    unmaximize?: () => Promise<void>;
    toggleMaximize?: () => Promise<void>;
    close?: () => Promise<void>;
    isMaximized?: () => Promise<boolean> | boolean;
    innerSize?: () => Promise<WindowPhysicalSize>;
    outerSize?: () => Promise<WindowPhysicalSize>;
    outerPosition?: () => Promise<WindowPhysicalPosition>;
    scaleFactor?: (() => Promise<number> | number) | number;
    setSize?: (size: WindowPhysicalSize) => Promise<void>;
    setPosition?: (position: WindowPhysicalPosition) => Promise<void>;
    setSizeConstraints?: (
        constraints: WindowSizeConstraints | null
    ) => Promise<void>;
    setMaximizable?: (maximizable: boolean) => Promise<void>;
    setAlwaysOnTop?: (alwaysOnTop: boolean) => Promise<void>;
    setFocus?: () => Promise<void>;
    requestUserAttention?: (requestType: number | null) => Promise<void>;
    setTheme?: (theme: WindowTheme | null) => Promise<void>;
};

/**
 * Browser build: window management belongs to the browser chrome. These
 * helpers keep the generated call surface compiling while performing the
 * closest web equivalent (or nothing).
 */
function createBrowserWindowStub(): WindowLike {
    const noop = async () => undefined;
    return {
        startDragging: noop,
        startResizeDragging: noop,
        minimize: noop,
        maximize: noop,
        unmaximize: noop,
        toggleMaximize: noop,
        close: async () => {
            window.close();
        },
        isMaximized: async () => false,
        innerSize: async () => ({
            width: window.innerWidth,
            height: window.innerHeight
        }),
        outerSize: async () => ({
            width: window.outerWidth,
            height: window.outerHeight
        }),
        outerPosition: async () => ({
            x: window.screenX,
            y: window.screenY
        }),
        scaleFactor: () => window.devicePixelRatio,
        setSize: noop,
        setPosition: noop,
        setSizeConstraints: noop,
        setMaximizable: noop,
        setAlwaysOnTop: noop,
        setFocus: noop,
        requestUserAttention: noop,
        setTheme: noop
    };
}

export async function getCurrentWebviewWindow(): Promise<WebviewWindowLike> {
    return createBrowserWindowStub() as unknown as WebviewWindowLike;
}

export async function getCurrentWindow(): Promise<WindowLike> {
    return createBrowserWindowStub();
}

export async function setZoom(zoom: number): Promise<void> {
    // Browser zoom is user-controlled; no-op for the app.
    void zoom;
    return undefined;
}

export async function getScaleFactor(): Promise<number | null> {
    try {
        return window.devicePixelRatio;
    } catch (error) {
        throw normalizePlatformError(error, 'Unable to read scale factor');
    }
}

export async function startDraggingWindow(): Promise<void> {
    return undefined;
}

export async function startResizeDraggingWindow(
    _direction: WindowResizeDirection
): Promise<void> {
    return undefined;
}

export async function minimizeWindow(): Promise<void> {
    return undefined;
}

export async function toggleMaximizeWindow(): Promise<void> {
    return undefined;
}

export async function maximizeWindow(): Promise<void> {
    return undefined;
}

export async function unmaximizeWindow(): Promise<void> {
    return undefined;
}

export async function closeWindow(): Promise<void> {
    window.close();
}

export async function focusWindow(): Promise<void> {
    window.focus();
}

export async function flashWindow(): Promise<void> {
    return undefined;
}

export async function setWindowTheme(
    _theme: WindowTheme | null
): Promise<void> {
    return undefined;
}

export async function isWindowMaximized(): Promise<boolean> {
    return false;
}

export async function getWindowGeometry(): Promise<WindowGeometry | null> {
    return {
        size: {
            width: window.outerWidth,
            height: window.outerHeight
        },
        position: { x: window.screenX, y: window.screenY },
        maximized: false
    };
}

export async function setWindowPhysicalPosition(
    _position: WindowPhysicalPosition
): Promise<void> {
    return undefined;
}

export async function setWindowBounds(_bounds: WindowBounds): Promise<void> {
    return undefined;
}

export async function setWindowSizeConstraints(
    _constraints: WindowSizeConstraints | null
): Promise<void> {
    return undefined;
}

export async function setWindowMaximizable(
    _maximizable: boolean
): Promise<void> {
    return undefined;
}

export async function setWindowAlwaysOnTop(
    _alwaysOnTop: boolean
): Promise<void> {
    return undefined;
}
