use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Event trait
pub trait Event: Any + Serialize + DeserializeOwned
                + Clone + Send + Sync {} // Events should be thread-safe

pub trait EventInstance: Any {
    fn serialize(&self) -> Value;
}

impl<T: Event> EventInstance for T {
    fn serialize(&self) -> Value {
        serde_json::to_value(self.clone()).unwrap()
    }
}

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
type ListenerFunction = dyn FnMut(&dyn EventInstance) -> BoxedFuture;
type BoxedListenerFunction = Box<ListenerFunction>;
type BoxedListenerVec = Vec<BoxedListenerFunction>;

pub struct EventDispatcher {
    listeners: RwLock<HashMap<TypeId, BoxedListenerVec>>
}

impl EventDispatcher {
    pub fn new() -> Arc<EventDispatcher> {
        Arc::new(EventDispatcher {
            listeners: Default::default()
        })
    }

    pub async fn add_listener<Ev, F, Fut>(&self, mut listener: F)
    where
        Ev: Event,
        F: FnMut(Ev) -> Fut + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let mut lock = self.listeners.write().await;
        let ty = TypeId::of::<Ev>();

        let listen_fn: BoxedListenerFunction = Box::new(move |event: &dyn EventInstance| {
            let event = unsafe {
                let ptr: *const dyn EventInstance = event;
                &*(ptr as *const Ev)
            };

            Box::pin(listener(event.clone()))
        });

        if let Some(listeners) = lock.get_mut(&ty) {
            listeners.push(listen_fn);
        } else {
            lock.insert(ty, vec![listen_fn]);
        }
    }

    pub async fn invoke<Ev: EventInstance + Event>(&self, event: Ev) {
        let mut lock = self.listeners.write().await;

        if let Some(listeners) = lock.get_mut(&event.type_id()) {
            for listener in listeners {
                listener(&event).await
            }
        }
    }
}