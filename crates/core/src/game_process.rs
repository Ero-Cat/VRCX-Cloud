#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameProcessEvent {
    pub is_game_running: bool,
    pub game_changed: bool,
}
