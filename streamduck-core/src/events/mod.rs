use std::any::Any;
use std::future::Future;
use std::sync::Arc;
use futures::future::join_all;
use futures::stream::FuturesUnordered;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::events::listeners::{EventListener, ListensFor, SharedEventListener, WeakEventListener};

/// Definitions for event listeners
pub mod listeners;
/// Util functions for dealing with events
pub mod util;

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
    listeners: Mutex<Vec<(ListensFor, WeakEventListener)>>
}

impl EventDispatcher {
    /// Creates a new dispatcher
    pub fn new() -> Arc<EventDispatcher> {
        Arc::new(EventDispatcher {
            listeners: Default::default()
        })
    }

    /// Adds listener into the dispatcher and returns its strong reference, listener is deleted once the strong reference gets dropped
    pub async fn add_listener<F>(&self, listener: F) -> SharedEventListener
    where
        F: EventListener + 'static,
    {
        let strong: SharedEventListener = Arc::new(listener);

        let weak: WeakEventListener = Arc::downgrade(&strong);

        self.add_listener_weak(strong.listens_for(), weak).await;

        strong
    }

    /// Adds weak reference of a listener into the dispatcher
    pub async fn add_listener_weak(&self, listens_for: ListensFor, weak: WeakEventListener) {
        let mut lock = self.listeners.lock().await;
        lock.push((listens_for, weak));
    }


    /// Invokes listeners with provided event
    pub async fn invoke<Ev: EventInstance + Event>(&self, event: Ev) {
        let mut lock = self.listeners.lock().await;

        lock.retain(|(_, l)| l.strong_count() > 0);

        let type_id = event.type_id();

        let listeners = lock.iter()
            .filter(|(l, _)| l & type_id)
            .map(|(_, weak)| async {
                if let Some(listener) = weak.upgrade() {
                    listener.invoke(&event).await
                }
            });

        join_all(listeners).await;
    }
}