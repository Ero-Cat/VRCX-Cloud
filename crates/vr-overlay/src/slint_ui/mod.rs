mod hmd;
mod platform;
mod surface;
mod wrist;

slint::include_modules!();

pub use hmd::SlintHmdHost;
pub use surface::{SlintHmdRenderer, SlintSurfaceHost, SlintSurfaceRenderer, SlintWristRenderer};
pub use wrist::SlintWristHost;

#[cfg(test)]
mod tests;
