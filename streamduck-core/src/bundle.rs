use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::devices::drivers::DriverManager;
use crate::events::EventDispatcher;

/// Bundle that contains all managers used by the core
pub struct ManagerBundle {
    pub(crate) driver_manager: OnceCell<Arc<DriverManager>>,
    pub(crate) global_dispatcher: OnceCell<Arc<EventDispatcher>>
}

impl ManagerBundle {
    /// Creates a new bundle
    pub fn new() -> Arc<ManagerBundle> {
        Arc::new(ManagerBundle {
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
}