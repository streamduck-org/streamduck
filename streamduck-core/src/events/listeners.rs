use std::any::TypeId;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::BitAnd;
use std::sync::{Arc, Weak};

use async_trait::async_trait;

use crate::events::{Event, EventInstance};
use crate::events::util::cast_event;
use crate::type_of;

/// Specifies which events listener is listening to. Can be used with bitwise AND (&) to compare if it listens for specified event
#[derive(Hash)]
pub enum ListensFor {
    /// Listens to any events
    Any,
    /// Listens to a single specific event
    Specific(TypeId),
    /// Listens to a list of events
    Multiple(Vec<TypeId>)
}

impl BitAnd<&TypeId> for &ListensFor {
    type Output = bool;

    fn bitand(self, rhs: &TypeId) -> Self::Output {
        match self {
            ListensFor::Any => true,
            ListensFor::Specific(ty) => *ty == *rhs,
            ListensFor::Multiple(list) => list.contains(rhs)
        }
    }
}

impl BitAnd<TypeId> for &ListensFor {
    type Output = bool;

    fn bitand(self, rhs: TypeId) -> Self::Output {
        self & (&rhs)
    }
}

impl BitAnd<&ListensFor> for &dyn EventInstance {
    type Output = bool;

    fn bitand(self, rhs: &ListensFor) -> Self::Output {
        rhs & self.type_id()
    }
}

impl BitAnd<&dyn EventInstance> for ListensFor {
    type Output = bool;

    fn bitand(self, rhs: &dyn EventInstance) -> Self::Output {
        (&self) & rhs.type_id()
    }
}

impl BitAnd<&dyn EventInstance> for &ListensFor {
    type Output = bool;

    fn bitand(self, rhs: &dyn EventInstance) -> Self::Output {
        (self) & rhs.type_id()
    }
}

/// Trait for types that want to accept events from event dispatcher
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Which events this listener should be invoked with
    fn listens_for(&self) -> ListensFor;

    /// Invokes listener with an event reference
    async fn invoke(&self, _: &dyn EventInstance);
}

/// Reference counted event listener
pub type SharedEventListener = Arc<dyn EventListener>;

/// Weak reference to an event listener
pub type WeakEventListener = Weak<dyn EventListener>;

/// General event listener
pub struct GeneralListener<F, Fut>
    where
        F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
        Fut: Future<Output = ()> + Send
{
    listener: F
}

impl<F, Fut> GeneralListener<F, Fut>
where
    F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    /// Creates a new listener from a closure
    pub fn new(listener: F) -> GeneralListener<F, Fut> {
        GeneralListener {
            listener
        }
    }
}

#[async_trait]
impl<F, Fut> EventListener for GeneralListener<F, Fut>
where
    F: Fn(&dyn EventInstance) -> Fut + Sync + Send,
    Fut: Future<Output = ()> + Send
{
    fn listens_for(&self) -> ListensFor {
        ListensFor::Any
    }

    async fn invoke(&self, event: &dyn EventInstance) {
        (self.listener)(event).await;
    }
}

/// Event listener that listens for specific event
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
    fn listens_for(&self) -> ListensFor {
        ListensFor::Specific(type_of!(Ev))
    }

    async fn invoke(&self, event: &dyn EventInstance) {
        if let Some(event) = cast_event::<Ev>(event) {
            (self.listener)(event).await;
        }
    }
}