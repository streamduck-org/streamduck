mod prompt;
mod helps;

use std::env;
use std::sync::Arc;
use streamduck_client::SDClient;
use streamduck_client::unix::UnixClient;
use crate::prompt::prompt;

fn main() {
    let args: Vec<String> = env::args().collect();

    let client = get_client(&args);

    if args.len() <= 1 {
        prompt(client);
    }
}

#[cfg(target_family = "unix")]
fn get_client(_args: &Vec<String>) -> Arc<Box<dyn SDClient>> {
    // TODO: Allow choosing connection method later
    UnixClient::new().expect("Failed to connect to daemon, is it up?")
}