use std::any::Any;
use std::future::Future;
use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::events::listeners::{EventListener, SharedEventListener, SpecificListener};

/// Definitions for event listeners
pub mod listeners;

/// Event trait
pub trait Event: Any + Serialize + DeserializeOwned
                + Clone + Send + Sync {} // Events should be thread-safe

/// Instance of an event
pub trait EventInstance: Any + Sync + Send {
    /// Serializes event into a JSON value
    fn serialize(&self) -> Value;
}

impl<T: Event> EventInstance for T {
    fn serialize(&self) -> Value {
        serde_json::to_value(self.clone()).unwrap()
    }
}

/// Event dispatcher
pub struct EventDispatcher {
    listeners: RwLock<Vec<SharedEventListener>>
}

impl EventDispatcher {
    /// Creates a new dispatcher
    pub fn new() -> Arc<EventDispatcher> {
        Arc::new(EventDispatcher {
            listeners: Default::default()
        })
    }

    /// Creates a closure based listener for specific event type
    pub async fn add_listener<F>(&self, listener: F)
    where
        F: EventListener + 'static,
    {
        let mut lock = self.listeners.write().await;

        lock.push(Arc::new(listener))
    }


    /// Invokes listeners with provided event
    pub async fn invoke<Ev: EventInstance + Event>(&self, event: Ev) {
        let lock = self.listeners.read().await;

        for listener in lock.iter() {
            listener.invoke(&event).await
        }
    }
}