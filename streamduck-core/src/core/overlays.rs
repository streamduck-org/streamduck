use serde::{Deserialize, Serialize};
use std::error::Error;
use std::hash::Hasher;
use std::sync::{Arc, Weak};
use async_trait::async_trait;
use image::DynamicImage;
use rmpv::Value;
use crate::config::SharedConfig;
use crate::core::Core;
use crate::data::{NamespacedName, Source};
use crate::plugin::Plugin;
use crate::ui::UISchema;

/// Shared reference to an overlay
pub type SharedOverlay = Arc<Overlay>;

/// Weak shared reference to an overlay
pub type WeakOverlay = Weak<Overlay>;

/// Image overlay over the button
pub struct Overlay {
    pub(crate) config: SharedConfig,

    /// Plugin that the overlay originated from
    pub original_plugin: Weak<Plugin>,

    /// Name of the overlay
    pub name: NamespacedName,

    /// Implementation of the overlay
    pub implement: Arc<dyn OverlayImpl>,

    /// UI Schema of the overlay
    pub ui: UISchema
}

/// Overlay implementation
#[async_trait]
pub trait OverlayImpl: Send + Sync  {
    /// Called when action options on some button got changed. Updated options are given along with new data separately
    async fn options_changed(&self, source: &Source, options: &Value, new_data: Value);

    /// Global tick
    async fn tick(&self, options: &Value, core: Arc<Core>);

    /// Asks overlay to add their state to the hash to check if frame update is needed
    fn hash(&self, source: &Source, options: &Value, hasher: &mut dyn Hasher);

    /// Asks the overlay to draw on the image
    fn draw(&self, source: &Source, options: &Value, image: &mut DynamicImage) -> Result<(), Box<dyn Error>>;
}

/// Data to be used by the overlays
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayData {
    /// Name of the overlay responsible for this
    pub name: NamespacedName,

    /// Options for the overlay
    pub options: Value
}