use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::devices::drivers::DriverManager;
use crate::events::EventDispatcher;
use crate::config::ConfigManager;

/// Bundle that contains all managers used by the core
pub struct ManagerBundle {
    pub(crate) driver_manager: OnceCell<Arc<DriverManager>>,
    pub(crate) global_dispatcher: OnceCell<Arc<EventDispatcher>>,
    pub(crate) config_manager: Arc<ConfigManager>
}

impl ManagerBundle {
    /// Creates a new bundle
    pub async fn new() -> Arc<ManagerBundle> {
        Arc::new(ManagerBundle {
            config_manager: ConfigManager::new(None).await.unwrap(),
            driver_manager: Default::default(),
            global_dispatcher: Default::default()
        })
    }

    /// Retrieves driver manager from the bundle
    pub fn driver_manager(&self) -> &Arc<DriverManager> {
        self.driver_manager.get()
            .expect("Driver manager wasn't initialized yet")
    }

    /// Retrieves global dispatcher from the bundle
    pub fn global_dispatcher(&self) -> &Arc<EventDispatcher> {
        self.global_dispatcher.get()
            .expect("Global dispatcher wasn't initialized yet")
    }

    /// Retrieves config manager from the bundle
    pub fn config_manager(&self) -> &Arc<ConfigManager> {
        &self.config_manager
    }
}
