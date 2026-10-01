use std::cell::RefCell;
use std::sync::Arc;
use windows_sys::Win32::Foundation::{ERROR_HOTKEY_ALREADY_REGISTERED, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT, VK_CONTROL, VK_LWIN, VK_MENU,
    VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

pub use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};

#[derive(Clone, Copy)]
pub struct CapturedKey {
    pub virtual_key: u32,
    pub modifiers: u32,
}

struct Recorder {
    hook: HHOOK,
    window: isize,
    pressed: [bool; 256],
    handler: CaptureHandler,
}

type CaptureHandler = Arc<dyn Fn(CapturedKey) + Send + Sync>;

enum HookAction {
    Forward,
    Consume,
    Record(CaptureHandler, CapturedKey),
}

impl Recorder {
    fn key_event(&mut self, virtual_key: u32, down: bool, modifiers: u32) -> HookAction {
        let Some(pressed) = self.pressed.get_mut(virtual_key as usize) else {
            return HookAction::Forward;
        };
        if !down {
            return if std::mem::take(pressed) {
                HookAction::Consume
            } else {
                HookAction::Forward
            };
        }
        if *pressed {
            return HookAction::Consume;
        }
        if modifiers & MOD_WIN != 0 || modifiers & (MOD_CONTROL | MOD_ALT) == 0 {
            return HookAction::Forward;
        }
        if !matches!(virtual_key, 0x30..=0x39 | 0x41..=0x5a | 0x70..=0x87) {
            return HookAction::Forward;
        }
        *pressed = true;
        HookAction::Record(
            self.handler.clone(),
            CapturedKey {
                virtual_key,
                modifiers,
            },
        )
    }
}

thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

/// Start and stop on the window's event-loop thread, which also receives the hook callbacks.
pub fn start(
    window: isize,
    handler: impl Fn(CapturedKey) + Send + Sync + 'static,
) -> Result<(), String> {
    stop();
    // SAFETY: the static callback lives in this executable and the event-loop thread remains alive.
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook),
            GetModuleHandleW(std::ptr::null()),
            0,
        )
    };
    if hook.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    RECORDER.with(|recorder| {
        *recorder.borrow_mut() = Some(Recorder {
            hook,
            window,
            pressed: [false; 256],
            handler: Arc::new(handler),
        });
    });
    Ok(())
}

pub fn stop() {
    let recorder = RECORDER.with(|recorder| recorder.borrow_mut().take());
    if let Some(recorder) = recorder {
        // SAFETY: the hook was installed on this thread; its callback does not retain this state.
        if unsafe { UnhookWindowsHookEx(recorder.hook) } == 0 {
            tracing::warn!(error = %std::io::Error::last_os_error(), "failed to remove shortcut recording hook");
        }
    }
}

pub fn code_name(virtual_key: u32) -> Option<String> {
    match virtual_key {
        0x30..=0x39 => Some(format!("Digit{}", char::from(virtual_key as u8))),
        0x41..=0x5a => Some(format!("Key{}", char::from(virtual_key as u8))),
        0x70..=0x87 => Some(format!("F{}", virtual_key - 0x70 + 1)),
        _ => None,
    }
}

fn virtual_key(code: &str) -> Option<u32> {
    if let Some(key) = code.strip_prefix("Key") {
        if key.len() == 1 && key.as_bytes()[0].is_ascii_uppercase() {
            return Some(u32::from(key.as_bytes()[0]));
        }
    }
    if let Some(key) = code.strip_prefix("Digit") {
        if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() {
            return Some(u32::from(key.as_bytes()[0]));
        }
    }
    code.strip_prefix('F')
        .and_then(|key| key.parse::<u32>().ok())
        .filter(|key| (1..=24).contains(key))
        .map(|key| 0x70 + key - 1)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    InUse,
    Unavailable(String),
}

pub fn check(code: &str, modifiers: u32) -> Result<(), ProbeError> {
    let key = virtual_key(code)
        .ok_or_else(|| ProbeError::Unavailable("Unsupported shortcut key".into()))?;
    const PROBE_ID: i32 = 0xbffe;
    // SAFETY: this temporary registration belongs to the calling thread and is released before returning.
    if unsafe {
        RegisterHotKey(
            std::ptr::null_mut(),
            PROBE_ID,
            modifiers | MOD_NOREPEAT,
            key,
        )
    } == 0
    {
        let error = std::io::Error::last_os_error();
        return Err(
            if error.raw_os_error() == Some(ERROR_HOTKEY_ALREADY_REGISTERED as i32) {
                ProbeError::InUse
            } else {
                ProbeError::Unavailable(error.to_string())
            },
        );
    }
    // SAFETY: this matches the temporary registration above on the same thread.
    if unsafe { UnregisterHotKey(std::ptr::null_mut(), PROBE_ID) } == 0 {
        return Err(ProbeError::Unavailable(
            std::io::Error::last_os_error().to_string(),
        ));
    }
    Ok(())
}

unsafe extern "system" fn keyboard_hook(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code < 0
        || !matches!(
            message as u32,
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP
        )
    {
        return CallNextHookEx(std::ptr::null_mut(), code, message, data);
    }
    // SAFETY: nonnegative keyboard hook callbacks supply a valid KBDLLHOOKSTRUCT for these messages.
    let event = &*(data as *const KBDLLHOOKSTRUCT);
    let action = RECORDER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(recorder) = slot.as_mut() else {
            return HookAction::Forward;
        };
        if GetForegroundWindow() as isize != recorder.window || event.vkCode >= 256 {
            return HookAction::Forward;
        }
        if matches!(message as u32, WM_KEYUP | WM_SYSKEYUP) {
            return recorder.key_event(event.vkCode, false, 0);
        }
        let mut modifiers = 0;
        if GetAsyncKeyState(i32::from(VK_CONTROL)) < 0 {
            modifiers |= MOD_CONTROL;
        }
        if GetAsyncKeyState(i32::from(VK_MENU)) < 0 {
            modifiers |= MOD_ALT;
        }
        if GetAsyncKeyState(i32::from(VK_SHIFT)) < 0 {
            modifiers |= MOD_SHIFT;
        }
        if GetAsyncKeyState(i32::from(VK_LWIN)) < 0 || GetAsyncKeyState(i32::from(VK_RWIN)) < 0 {
            modifiers |= MOD_WIN;
        }
        recorder.key_event(event.vkCode, true, modifiers)
    });
    match action {
        HookAction::Record(handler, event) => {
            handler(event);
            1
        }
        HookAction::Consume => 1,
        HookAction::Forward => CallNextHookEx(std::ptr::null_mut(), code, message, data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_key_codes_round_trip() {
        for key in (0x30..=0x39).chain(0x41..=0x5a).chain(0x70..=0x87) {
            assert_eq!(virtual_key(&code_name(key).unwrap()), Some(key));
        }
        assert!(code_name(u32::from(VK_CONTROL)).is_none());
    }

    #[test]
    fn recording_ignores_windows_keys_and_plain_typing_but_consumes_recorded_key_pairs() {
        let mut recorder = Recorder {
            hook: std::ptr::null_mut(),
            window: 0,
            pressed: [false; 256],
            handler: Arc::new(|_| {}),
        };
        assert!(matches!(
            recorder.key_event(0x4b, true, MOD_WIN | MOD_SHIFT),
            HookAction::Forward
        ));
        assert!(matches!(
            recorder.key_event(0x4d, true, 0),
            HookAction::Forward
        ));
        assert!(matches!(
            recorder.key_event(0x4d, true, MOD_SHIFT),
            HookAction::Forward
        ));
        assert!(matches!(
            recorder.key_event(0x4d, true, MOD_CONTROL | MOD_SHIFT),
            HookAction::Record(_, _)
        ));
        assert!(matches!(
            recorder.key_event(0x4d, true, MOD_CONTROL | MOD_SHIFT),
            HookAction::Consume
        ));
        assert!(matches!(
            recorder.key_event(0x4d, false, 0),
            HookAction::Consume
        ));
        assert!(matches!(
            recorder.key_event(0x4d, false, 0),
            HookAction::Forward
        ));
    }

    #[test]
    #[ignore = "Registers a temporary system-wide hotkey; run explicitly in a desktop session"]
    fn native_probe_reports_occupied_keys_and_releases_available_keys() {
        const OWNER_ID: i32 = 0xbffd;
        struct Registration;
        impl Drop for Registration {
            fn drop(&mut self) {
                // SAFETY: the test registration belongs to this thread.
                unsafe {
                    UnregisterHotKey(std::ptr::null_mut(), OWNER_ID);
                }
            }
        }
        let modifiers = MOD_CONTROL | MOD_ALT | MOD_SHIFT;
        let key = (0x7c..=0x87)
            .find(|key| {
                // SAFETY: this uses a temporary thread-owned hotkey, with no external window handle.
                unsafe {
                    RegisterHotKey(
                        std::ptr::null_mut(),
                        OWNER_ID,
                        modifiers | MOD_NOREPEAT,
                        *key,
                    ) != 0
                }
            })
            .expect("a free function-key combination for the native probe test");
        let registration = Registration;
        let code = code_name(key).unwrap();
        assert_eq!(check(&code, modifiers), Err(ProbeError::InUse));
        drop(registration);
        assert_eq!(check(&code, modifiers), Ok(()));
        assert_eq!(check(&code, modifiers), Ok(()));
    }
}
