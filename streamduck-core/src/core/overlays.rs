use std::error::Error;
use std::sync::Weak;
use async_trait::async_trait;
use image::DynamicImage;
use rmpv::Value;
use crate::data::Source;
use crate::plugin::Plugin;
use crate::ui::UISchema;

/// Image overlay over the button
pub struct Overlay {
    /// Plugin that the overlay originated from
    pub original_plugin: Weak<Plugin>,

    /// Name of the overlay
    pub name: String,

    /// Implementation of the overlay
    pub implement: Box<dyn OverlayImpl>,

    /// UI Schema of the overlay
    pub ui: UISchema
}

/// Overlay implementation
#[async_trait]
pub trait OverlayImpl {
    /// Called when action options on some button got changed. Updated options are given along with new data separately
    async fn options_changed(&self, source: &Source, options: &Value, new_data: Value);

    /// Asks the overlay to draw on the image
    async fn draw(&self, source: &Source, options: &Value, image: &mut DynamicImage) -> OverlayResult;
}

/// Result of overlay draw
#[derive(Debug)]
pub enum OverlayResult {
    /// Overlay doesn't need to perform any changes to the image
    NoChanges,

    /// Overlay modified the image
    Changed,

    /// Overlay errored out
    Error(Box<dyn Error>)
}