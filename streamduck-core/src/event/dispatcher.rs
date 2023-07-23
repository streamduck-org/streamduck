use std::sync::{Arc, Weak};
use scc::Queue;
use tokio::sync::{Mutex, Notify};
use crate::event::Event;

/// Shared reference to the Event Dispatcher
pub type SharedEventDispatcher = Arc<EventDispatcher>;

/// Dispatches incoming events to various subscribers
#[derive(Default)]
pub struct EventDispatcher {
    listeners: Mutex<Vec<WeakSubscriber>>
}

impl EventDispatcher {
    /// Creates new dispatcher
    pub fn new() -> SharedEventDispatcher {
        Arc::new(EventDispatcher {
            listeners: Default::default()
        })
    }

    /// Returns subscriber that can be used to get events asynchronously
    pub async fn subscribe(&self) -> Arc<Subscriber> {
        let subscriber = Arc::new(Subscriber::default());
        self.listeners.lock().await.push(Arc::downgrade(&subscriber));
        subscriber
    }

    /// Sends an event to all subscribers
    pub async fn send(&self, event: Event) {
        let mut lock = self.listeners.lock().await;

        let mut new_subscribers = Vec::new();

        for subscriber in lock.iter().filter_map(|x| x.upgrade()) {
            subscriber.events.push(event.clone());
            subscriber.notify.notify_one();

            new_subscribers.push(Arc::downgrade(&subscriber));
        }

        *lock = new_subscribers;
    }
}

/// Shared reference to the Subscriber
pub type SharedSubscriber = Arc<Subscriber>;

/// Weak reference to the Subscriber
pub type WeakSubscriber = Weak<Subscriber>;

/// Subscriber to an event dispatcher
#[derive(Default)]
pub struct Subscriber {
    notify: Notify,
    events: Queue<Event>
}

impl Subscriber {
    /// Awaits for an event
    #[async_recursion::async_recursion]
    pub async fn get(&self) -> Event {
        if let Some(event) = self.events.pop() {
            (**event).clone()
        } else {
            self.notify.notified().await;
            self.get().await
        }
    }
}