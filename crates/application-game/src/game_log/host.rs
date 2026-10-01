use crate::Result;

pub trait GameLogHostActions: Send + Sync {
    fn quit_game(&self) -> i64;
    fn copy_image_to_clipboard(&self, path: &str) -> Result<()>;
    fn ugc_photo_location(&self, configured_path: Option<String>) -> String;
    fn add_screenshot_metadata(
        &self,
        path: &str,
        metadata: &str,
        world_id: &str,
        modify_filename: bool,
    ) -> String;
}

#[derive(Default)]
pub struct NoopGameLogHostActions;

impl GameLogHostActions for NoopGameLogHostActions {
    fn quit_game(&self) -> i64 {
        0
    }

    fn copy_image_to_clipboard(&self, _path: &str) -> Result<()> {
        Ok(())
    }

    fn ugc_photo_location(&self, configured_path: Option<String>) -> String {
        configured_path.unwrap_or_default()
    }

    fn add_screenshot_metadata(
        &self,
        path: &str,
        _metadata: &str,
        _world_id: &str,
        _modify_filename: bool,
    ) -> String {
        path.to_string()
    }
}
