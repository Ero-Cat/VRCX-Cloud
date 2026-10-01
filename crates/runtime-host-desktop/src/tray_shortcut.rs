#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TrayShortcutBinding {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: String,
}

impl TrayShortcutBinding {
    pub fn validate(&self) -> Result<(), TrayShortcutError> {
        let letter = self
            .key
            .strip_prefix("Key")
            .is_some_and(|key| key.len() == 1 && key.as_bytes()[0].is_ascii_uppercase());
        let digit = self
            .key
            .strip_prefix("Digit")
            .is_some_and(|key| key.len() == 1 && key.as_bytes()[0].is_ascii_digit());
        let function = self.key.strip_prefix('F').is_some_and(|key| {
            key.parse::<u8>()
                .is_ok_and(|number| (1..=24).contains(&number) && key == number.to_string())
        });
        if !(self.control || self.alt) || !(letter || digit || function) {
            return Err(TrayShortcutError::Invalid);
        }
        if self.key == "F12"
            || (self.control
                && !self.alt
                && (matches!(self.key.as_str(), "KeyB" | "KeyD" | "KeyK")
                    || (!self.shift
                        && matches!(
                            self.key.as_str(),
                            "KeyA" | "KeyC" | "KeyV" | "KeyX" | "KeyY" | "KeyZ" | "KeyP"
                        ))))
        {
            return Err(TrayShortcutError::Reserved);
        }
        Ok(())
    }

    pub fn accelerator(&self) -> String {
        let mut keys = Vec::with_capacity(4);
        if self.control {
            keys.push("Control");
        }
        if self.alt {
            keys.push("Alt");
        }
        if self.shift {
            keys.push("Shift");
        }
        keys.push(&self.key);
        keys.join("+")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TrayShortcutError {
    Invalid,
    Reserved,
    InUse,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TrayShortcutStatus {
    Unset,
    Active,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TrayShortcutSnapshot {
    pub binding: Option<TrayShortcutBinding>,
    pub status: TrayShortcutStatus,
}

#[derive(serde::Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TrayShortcutUpdate {
    Saved {
        snapshot: TrayShortcutSnapshot,
    },
    Failed {
        error: TrayShortcutError,
        snapshot: TrayShortcutSnapshot,
    },
}

pub trait TrayShortcutRegistrar {
    fn register(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError>;
    fn unregister(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError>;

    fn check(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError>;
}

pub struct TrayShortcutRuntime {
    snapshot: TrayShortcutSnapshot,
}

impl TrayShortcutRuntime {
    pub fn new(binding: Option<TrayShortcutBinding>, supported: bool) -> Self {
        let status = if !supported {
            TrayShortcutStatus::Unsupported
        } else if binding.is_some() {
            TrayShortcutStatus::Unavailable
        } else {
            TrayShortcutStatus::Unset
        };
        Self {
            snapshot: TrayShortcutSnapshot { binding, status },
        }
    }

    pub fn snapshot(&self) -> TrayShortcutSnapshot {
        self.snapshot.clone()
    }

    pub fn check(
        &self,
        binding: &TrayShortcutBinding,
        registrar: &impl TrayShortcutRegistrar,
    ) -> Result<(), TrayShortcutError> {
        if self.snapshot.status == TrayShortcutStatus::Unsupported {
            return Err(TrayShortcutError::Unsupported);
        }
        binding.validate()?;
        if self.snapshot.status == TrayShortcutStatus::Active
            && self.snapshot.binding.as_ref() == Some(binding)
        {
            return Ok(());
        }
        registrar.check(binding)
    }

    pub fn configure(
        &mut self,
        binding: Option<TrayShortcutBinding>,
        registrar: &impl TrayShortcutRegistrar,
    ) -> Result<TrayShortcutSnapshot, TrayShortcutError> {
        if self.snapshot.status == TrayShortcutStatus::Unsupported {
            if binding.is_some() {
                return Err(TrayShortcutError::Unsupported);
            }
            self.snapshot.binding = None;
            return Ok(self.snapshot());
        }
        if let Some(next) = &binding {
            next.validate()?;
        }
        if self.snapshot.status == TrayShortcutStatus::Active && self.snapshot.binding == binding {
            return Ok(self.snapshot());
        }
        if let Some(next) = &binding {
            registrar.register(next)?;
        }
        if self.snapshot.status == TrayShortcutStatus::Active {
            if let Some(previous) = &self.snapshot.binding {
                if let Err(error) = registrar.unregister(previous) {
                    if let Some(next) = &binding {
                        if let Err(cleanup_error) = registrar.unregister(next) {
                            tracing::warn!(
                                ?cleanup_error,
                                "failed to release the rejected tray shortcut"
                            );
                        }
                    }
                    return Err(error);
                }
            }
        }
        self.snapshot.status = if binding.is_some() {
            TrayShortcutStatus::Active
        } else {
            TrayShortcutStatus::Unset
        };
        self.snapshot.binding = binding;
        Ok(self.snapshot())
    }
}

pub struct TrayShortcutWindowState {
    pub visible: bool,
    pub minimized: bool,
    pub focused: bool,
    pub edge_hidden: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrayShortcutWindowAction {
    Show,
    Hide,
}

pub fn tray_shortcut_window_action(
    window: Option<TrayShortcutWindowState>,
) -> TrayShortcutWindowAction {
    match window {
        Some(window)
            if window.visible && window.focused && !window.minimized && !window.edge_hidden =>
        {
            TrayShortcutWindowAction::Hide
        }
        _ => TrayShortcutWindowAction::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashSet;

    #[derive(Default)]
    struct Registrar {
        registered: RefCell<HashSet<String>>,
        calls: RefCell<Vec<String>>,
        reject_unregister: RefCell<Option<String>>,
    }

    impl TrayShortcutRegistrar for Registrar {
        fn check(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError> {
            self.register(binding)?;
            self.unregister(binding)
        }

        fn register(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError> {
            self.calls
                .borrow_mut()
                .push(format!("register:{}", binding.key));
            if self.registered.borrow_mut().insert(binding.accelerator()) {
                Ok(())
            } else {
                Err(TrayShortcutError::Unavailable)
            }
        }

        fn unregister(&self, binding: &TrayShortcutBinding) -> Result<(), TrayShortcutError> {
            self.calls
                .borrow_mut()
                .push(format!("unregister:{}", binding.key));
            if self.reject_unregister.borrow().as_deref() == Some(&binding.key) {
                return Err(TrayShortcutError::Unavailable);
            }
            self.registered.borrow_mut().remove(&binding.accelerator());
            Ok(())
        }
    }

    fn binding(key: &str) -> TrayShortcutBinding {
        TrayShortcutBinding {
            control: true,
            alt: true,
            shift: false,
            key: key.into(),
        }
    }

    #[test]
    fn replacement_conflict_preserves_the_previous_binding_and_registration() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(None, true);
        runtime
            .configure(Some(binding("KeyV")), &registrar)
            .unwrap();
        registrar.register(&binding("KeyM")).unwrap();
        let before = runtime.snapshot();

        assert_eq!(
            runtime.configure(Some(binding("KeyM")), &registrar),
            Err(TrayShortcutError::Unavailable)
        );
        assert_eq!(runtime.snapshot(), before);
        assert!(registrar
            .registered
            .borrow()
            .contains(&binding("KeyV").accelerator()));
    }

    #[test]
    fn checking_does_not_change_the_configured_binding_or_leave_a_new_registration() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(None, true);
        let original = binding("KeyV");
        runtime
            .configure(Some(original.clone()), &registrar)
            .unwrap();
        let before = runtime.snapshot();
        let calls = registrar.calls.borrow().len();

        runtime.check(&original, &registrar).unwrap();
        assert_eq!(registrar.calls.borrow().len(), calls);
        runtime.check(&binding("KeyM"), &registrar).unwrap();
        assert_eq!(runtime.snapshot(), before);
        assert_eq!(
            *registrar.registered.borrow(),
            HashSet::from([original.accelerator()])
        );

        registrar.register(&binding("KeyM")).unwrap();
        assert_eq!(
            runtime.check(&binding("KeyM"), &registrar),
            Err(TrayShortcutError::Unavailable)
        );
        assert_eq!(runtime.snapshot(), before);
    }

    #[test]
    fn replacement_registers_the_new_key_before_releasing_the_old_one_and_clear_releases_it() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(None, true);
        runtime
            .configure(Some(binding("KeyV")), &registrar)
            .unwrap();
        runtime
            .configure(Some(binding("KeyM")), &registrar)
            .unwrap();
        runtime
            .configure(Some(binding("KeyM")), &registrar)
            .unwrap();
        runtime.configure(None, &registrar).unwrap();

        assert_eq!(
            *registrar.calls.borrow(),
            [
                "register:KeyV",
                "register:KeyM",
                "unregister:KeyV",
                "unregister:KeyM"
            ]
        );
        assert!(registrar.registered.borrow().is_empty());
        assert_eq!(runtime.snapshot().status, TrayShortcutStatus::Unset);
        assert!(runtime.snapshot().binding.is_none());
    }

    #[test]
    fn startup_conflict_keeps_the_saved_binding_for_retry() {
        let registrar = Registrar::default();
        let saved = binding("KeyV");
        registrar.register(&saved).unwrap();
        let mut runtime = TrayShortcutRuntime::new(Some(saved.clone()), true);

        assert_eq!(
            runtime.configure(Some(saved.clone()), &registrar),
            Err(TrayShortcutError::Unavailable)
        );
        assert_eq!(runtime.snapshot().binding, Some(saved.clone()));
        assert_eq!(runtime.snapshot().status, TrayShortcutStatus::Unavailable);

        registrar.unregister(&saved).unwrap();
        runtime.configure(Some(saved), &registrar).unwrap();
        assert_eq!(runtime.snapshot().status, TrayShortcutStatus::Active);
    }

    #[test]
    fn release_failure_rolls_back_the_new_registration() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(None, true);
        runtime
            .configure(Some(binding("KeyV")), &registrar)
            .unwrap();
        *registrar.reject_unregister.borrow_mut() = Some("KeyV".into());

        assert_eq!(
            runtime.configure(Some(binding("KeyM")), &registrar),
            Err(TrayShortcutError::Unavailable)
        );
        assert_eq!(runtime.snapshot().binding, Some(binding("KeyV")));
        assert_eq!(
            *registrar.registered.borrow(),
            HashSet::from([binding("KeyV").accelerator()])
        );
    }

    #[test]
    fn invalid_and_reserved_combinations_never_reach_the_registrar() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(None, true);
        for key in [
            "",
            "ControlLeft",
            "KeyVV",
            "Keyé",
            "F0",
            "F25",
            "F01",
            "F12",
        ] {
            assert!(runtime.configure(Some(binding(key)), &registrar).is_err());
        }
        let mut plain = binding("KeyV");
        plain.control = false;
        plain.alt = false;
        plain.shift = true;
        assert_eq!(
            runtime.configure(Some(plain), &registrar),
            Err(TrayShortcutError::Invalid)
        );
        let mut copy = binding("KeyC");
        copy.alt = false;
        assert_eq!(
            runtime.configure(Some(copy), &registrar),
            Err(TrayShortcutError::Reserved)
        );
        assert!(registrar.calls.borrow().is_empty());
    }

    #[test]
    fn unsupported_hosts_cannot_register_but_can_clear_a_saved_binding() {
        let registrar = Registrar::default();
        let mut runtime = TrayShortcutRuntime::new(Some(binding("KeyV")), false);
        assert_eq!(
            runtime.configure(Some(binding("KeyV")), &registrar),
            Err(TrayShortcutError::Unsupported)
        );
        runtime.configure(None, &registrar).unwrap();
        assert_eq!(runtime.snapshot().status, TrayShortcutStatus::Unsupported);
        assert!(runtime.snapshot().binding.is_none());
        assert!(registrar.calls.borrow().is_empty());
    }

    #[test]
    fn only_a_focused_expanded_visible_window_is_hidden() {
        assert_eq!(
            tray_shortcut_window_action(None),
            TrayShortcutWindowAction::Show
        );
        for (visible, minimized, focused, edge_hidden, expected) in [
            (true, false, true, false, TrayShortcutWindowAction::Hide),
            (true, false, false, false, TrayShortcutWindowAction::Show),
            (true, true, true, false, TrayShortcutWindowAction::Show),
            (false, false, true, false, TrayShortcutWindowAction::Show),
            (true, false, true, true, TrayShortcutWindowAction::Show),
        ] {
            assert_eq!(
                tray_shortcut_window_action(Some(TrayShortcutWindowState {
                    visible,
                    minimized,
                    focused,
                    edge_hidden
                })),
                expected
            );
        }
    }
}
