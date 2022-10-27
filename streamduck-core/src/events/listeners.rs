use std::any::TypeId;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use crate::events::{Event, EventInstance};

use async_trait::async_trait;

/// Trait for types that want to accept events from event dispatcher
#[async_trait]
pub trait EventListener {
    /// Invokes listener with an event reference
    async fn invoke(&self, _: &dyn EventInstance);
}

/// Reference counted event listener
pub type SharedEventListener = Arc<dyn EventListener>;

/// Stateless event listener
pub struct StatelessListener<F, Fut>
    where
        F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    listener: F
}

impl<F, Fut> StatelessListener<F, Fut>
where
    F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    /// Creates a new listener from state and a closure
    pub fn new(listener: F) -> StatelessListener<F, Fut> {
        StatelessListener {
            listener
        }
    }
}

#[async_trait]
impl<F, Fut> EventListener for StatelessListener<F, Fut>
where
    F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    async fn invoke(&self, event: &dyn EventInstance) {
        (self.listener)(event).await;
    }
}

/// Event listener that keeps its state
pub struct StatefulListener<S, F, Fut>
    where
        S: Sync + Send,
        F: Fn(&S, &dyn EventInstance) -> Fut + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    state: S,
    listener: F
}

impl<S, F, Fut> StatefulListener<S, F, Fut>
    where
        S: Sync + Send,
        F: Fn(&S, &dyn EventInstance) -> Fut + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    /// Creates a new listener from state and a closure
    pub fn new(state: S, listener: F) -> StatefulListener<S, F, Fut> {
        StatefulListener {
            state,
            listener
        }
    }
}

#[async_trait]
impl<T, F, Fut> EventListener for StatefulListener<T, F, Fut>
    where
        T: Sync + Send,
        F: Fn(&T, &dyn EventInstance) -> Fut + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    async fn invoke(&self, event: &dyn EventInstance) {
        (self.listener)(&self.state, event).await;
    }
}

/// Stateless event listener that listens for specific event
pub struct SpecificListener<Ev, F, Fut>
    where
        Ev: Event,
        F: Fn(Ev) -> Fut + 'static + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    listener: F,
    _phantom_event: PhantomData<Ev>
}

impl<Ev, F, Fut> SpecificListener<Ev, F, Fut>
where
    Ev: Event,
    F: Fn(Ev) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    /// Creates a new listener from a closure
    pub fn new(listener: F) -> SpecificListener<Ev, F, Fut> {
        SpecificListener {
            listener,
            _phantom_event: Default::default()
        }
    }
}

#[async_trait]
impl<Ev, F, Fut> EventListener for SpecificListener<Ev, F, Fut>
where
    Ev: Event,
    F: Fn(Ev) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    async fn invoke(&self, event: &dyn EventInstance) {
        if event.type_id() == TypeId::of::<Ev>() {
            let event = unsafe {
                let ptr: *const dyn EventInstance = event;
                (&*(ptr as *const Ev)).clone()
            };

            (self.listener)(event).await;
        }
    }
}

/// Event listener that listens for specific event and keeps its state
pub struct SpecificStatefulListener<S, Ev, F, Fut>
where
    Ev: Event,
    S: Sync + Send,
    F: Fn(&S, Ev) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    state: S,
    listener: F,
    _phantom_event: PhantomData<Ev>
}

impl<S, Ev, F, Fut> SpecificStatefulListener<S, Ev, F, Fut>
where
    Ev: Event,
    S: Sync + Send,
    F: Fn(&S, Ev) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    /// Creates a new listener from a closure
    pub fn new(state: S, listener: F) -> SpecificStatefulListener<S, Ev, F, Fut> {
        SpecificStatefulListener {
            state,
            listener,
            _phantom_event: Default::default()
        }
    }
}

#[async_trait]
impl<S, Ev, F, Fut> EventListener for SpecificStatefulListener<S, Ev, F, Fut>
where
    Ev: Event,
    S: Sync + Send,
    F: Fn(&S, Ev) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    async fn invoke(&self, event: &dyn EventInstance) {
        if event.type_id() == TypeId::of::<Ev>() {
            let event = unsafe {
                let ptr: *const dyn EventInstance = event;
                (&*(ptr as *const Ev)).clone()
            };

            (self.listener)(&self.state, event).await;
        }
    }
}