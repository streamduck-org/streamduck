use std::sync::Arc;
use tracing::{Level, info};
use streamduck_core::devices::drivers::DriverManager;
use streamduck_core::devices::images::DeviceImageCache;
use streamduck_core::events::{Event, EventDispatcher, EventInstance};
use serde::{Serialize, Deserialize};

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

    dispatcher.add_listener(|ev: MyEventButDifferent| {
        async move {
            println!("what? {}, {}", ev.a, ev.b);
        }
    }).await;

    // let listener = Listener {
    //     driver_manager: driver_manager.clone()
    // };
    //
    // dispatcher.add_listener(move |event: MyEvent| listener.listen(event)).await;


    println!("{:?}", driver_manager.list_devices().await);

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

impl Listener {
    async fn listen(&self, event: MyEvent) {
        println!("what????? {} {:?}", event.a, self.driver_manager.list_devices().await);
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