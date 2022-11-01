use std::future::ready;
use std::sync::Arc;
use tracing::{Level, info};
use streamduck_core::devices::drivers::DriverManager;
use streamduck_core::devices::images::DeviceImageCache;
use streamduck_core::events::{Event, EventDispatcher, EventInstance};
use serde::{Serialize, Deserialize};
use streamduck_core::events::listeners::{EventListener, ListensFor, SpecificListener};
use streamduck_core::events::util::cast_event;
use streamduck_core::type_of;

/// the entry point for the streamdeck application
#[tokio::main]
async fn main() {
    // TODO: change filter level depending on flag
    tracing_subscriber::fmt()
        .compact()
        .with_target(true)
        .with_max_level(Level::TRACE)
        .init();

    info!("Starting...");

    let driver_manager = DriverManager::new();

    let a: Option<Arc<dyn DeviceImageCache>> = None;

    let dispatcher = EventDispatcher::new();

    let _listener = dispatcher.add_listener(Listener { driver_manager: driver_manager.clone() }).await;

    dispatcher.invoke(MyEvent {
        a: 5
    }).await;

    dispatcher.invoke(MyEventButDifferent {
        a: "what th fk".to_string(),
        b: 5
    }).await;
}

struct Listener {
    driver_manager: Arc<DriverManager>
}

#[streamduck_core::async_trait]
impl EventListener for Listener {
    fn listens_for(&self) -> ListensFor {
        ListensFor::Specific(type_of!(MyEvent))
    }

    async fn invoke(&self, event: &dyn EventInstance) {
        if let Some(event) = cast_event::<MyEvent>(event) {
            println!("lol kekw {}", event.a);
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct MyEvent {
    pub a: i32
}

impl Event for MyEvent {}

#[derive(Clone, Serialize, Deserialize)]
struct MyEventButDifferent {
    pub a: String,
    pub b: i32,
}

impl Event for MyEventButDifferent {}