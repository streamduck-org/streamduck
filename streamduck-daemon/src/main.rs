mod streamdeck;

use std::time::Duration;
use tokio::runtime::Builder;
use tokio::task;
use tokio::time::sleep;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use streamduck_core::{Streamduck};
use crate::streamdeck::make_streamdeck_plugin;

/// Initializing tokio runtime
fn main() {
    Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async_main())
}

/// Actual main function
async fn async_main() {
    // Tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Couldn't setup tracing");


    let streamduck = Streamduck::init().await;

    streamduck.load_plugin(make_streamdeck_plugin(streamduck.get_config()).await.unwrap()).await;

    let cloned_streamduck = streamduck.clone();
    task::spawn(async move {
        sleep(Duration::from_secs_f32(2.0)).await;

        cloned_streamduck.refresh_devices().await;

        for device in cloned_streamduck.list_devices().await {
            cloned_streamduck.add_device_to_autoconnect(&device).await;
        }
    });

    streamduck.run().await
}