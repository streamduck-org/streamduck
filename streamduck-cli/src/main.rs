mod prompt;
mod helps;

use std::env;
use std::sync::Arc;
use streamduck_client::SDSyncRequestClient;
use crate::prompt::prompt;

fn main() {
    let args: Vec<String> = env::args().collect();

    let client = get_client(&args);

    if args.len() <= 1 {
        prompt(client);
    }
}

#[cfg(target_family = "windows")]
fn get_client(_args: &Vec<String>) -> Arc<dyn SDSyncRequestClient> {
    streamduck_client::windows::WinClient::new().expect("Failed to connect to daemon, is it up?").as_request()
}

#[cfg(target_family = "unix")]
fn get_client(_args: &Vec<String>) -> Arc<dyn SDSyncRequestClient> {
    streamduck_client::unix::UnixClient::new().expect("Failed to connect to daemon, is it up?").as_request()
}