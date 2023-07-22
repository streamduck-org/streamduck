mod streamdeck;

use std::time::Duration;
use base64::Engine;
use rmpv::Value;
use tokio::runtime::Builder;
use tokio::task;
use tokio::time::sleep;
use streamduck_core::data::Number;
use streamduck_core::{msgslice, msgvec, Streamduck};
use streamduck_core::trigger::{Condition, TriggerCondition};
use streamduck_core::ui::{Field, FieldCondition, FieldType, UISchema, LodashValuePath};
use streamduck_core::util::{traverse_msgpack, traverse_msgpack_mut};
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
    let streamduck = Streamduck::init().await;

    streamduck.load_plugin(make_streamdeck_plugin(streamduck.get_config()).await.unwrap()).await;

    let cloned_streamduck = streamduck.clone();
    task::spawn(async move {
        sleep(Duration::from_secs_f32(5.0)).await;

        cloned_streamduck.refresh_devices().await;

        for device in cloned_streamduck.list_devices().await {
            println!("device {:#?} has following metadata: \n{:#?}", device, cloned_streamduck.describe_device(&device).await);
        }
    });

    streamduck.run().await
}