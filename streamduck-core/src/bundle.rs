use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use hidapi::HidError;

use crate::config::{ConfigError, ConfigManager};
use crate::devices::drivers::DriverManager;
use crate::events::EventDispatcher;
use crate::localization::LocalizationManager;

/// Bundle that contains all managers used by the core
pub struct ManagerBundle {
    pub(crate) driver_manager: Arc<DriverManager>,
    pub(crate) global_dispatcher: Arc<EventDispatcher>,
    pub(crate) config_manager: Arc<ConfigManager>,
    pub(crate) localization_manager: Arc<LocalizationManager>,
}

impl ManagerBundle {
    /// Creates a new bundle
    pub async fn new() -> Result<Arc<ManagerBundle>, ManagerInitError> {
        Ok(Arc::new(ManagerBundle {
            config_manager: ConfigManager::new(None).await?,
            driver_manager: DriverManager::new()?,
            global_dispatcher: EventDispatcher::new(),
            localization_manager: LocalizationManager::new(),
        }))
    }

    /// Retrieves driver manager from the bundle
    pub fn driver_manager(&self) -> &Arc<DriverManager> {
        &self.driver_manager
    }

    /// Retrieves global dispatcher from the bundle
    pub fn global_dispatcher(&self) -> &Arc<EventDispatcher> {
        &self.global_dispatcher
    }

    /// Retrieves config manager from the bundle
    pub fn config_manager(&self) -> &Arc<ConfigManager> {
        &self.config_manager
    }

    /// Retrieves localization manager from the bundle
    pub fn localization_manager(&self) -> &Arc<LocalizationManager> {
        &self.localization_manager
    }
}

/// Error that might be returned by the managers as they init
#[derive(Debug)]
pub enum ManagerInitError {
    /// Failed to set value to OnceCell
    FailedOnceCell,

    /// Error that happened while initializing HidApi
    HidError(HidError),

    /// Config related error
    ConfigError(ConfigError),
}

impl Display for ManagerInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ManagerInitError {}

impl From<HidError> for ManagerInitError {
    fn from(e: HidError) -> Self {
        Self::HidError(e)
    }
}

impl From<ConfigError> for ManagerInitError {
    fn from(e: ConfigError) -> Self {
        Self::ConfigError(e)
    }
}
