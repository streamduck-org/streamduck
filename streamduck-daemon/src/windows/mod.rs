use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;
use named_pipe::{PipeOptions, PipeServer};
use streamduck_core::socket::SocketManager;
use std::io::Write;
use std::ops::Deref;

const PIPE_NAME: &'static str = "\\\\.\\pipe\\streamduck";

pub fn open_socket(socket_manager: Arc<SocketManager>) {
    loop {
        let instance = PipeOptions::new(PIPE_NAME)
            .first(false)
            .single().expect("Failed to create named pipe server");

        if let Ok(client) = instance.wait() {
            let manager = socket_manager.clone();
            spawn(move || handle_client(client, manager));
        }
    }
}

fn handle_client(mut client: PipeServer, manager: Arc<SocketManager>) {
    log::info!("Windows pipe client connected");

    write!(client, "helo there\u{0004}");

    let mut stream = BufReader::new(client);

    let write_ref = unsafe {
        let const_ptr = stream.get_ref() as *const PipeServer;
        let mut_ptr = const_ptr as *mut PipeServer;
        &mut *mut_ptr
    };

    let flag = Arc::new(RwLock::new(true));
    let write_flag = flag.clone();

    spawn(move || {
        let client = write_ref;

        while *write_flag.read().unwrap() {
            write!(client, "what\u{0004}");
            sleep(Duration::from_secs_f32(1.0))
        }
    });

    let mut message = vec![];
    while let Ok(size) = stream.read_until(0x4, &mut message) {
        if size <= 0 {
            break;
        }

        if let Ok(message) = String::from_utf8(message.clone()) {
            log::info!("message {}", message.replace("\u{0004}", ""));
            // match serde_json::from_str(&message.replace("\u{0004}", "")) {
            //     Ok(packet) => socket_manager.received_message(stream.get_mut(), packet),
            //     Err(e) => log::warn!("Invalid message in sockets: {}", e)
            // }
        }

        message.clear();
    }

    *flag.write().unwrap() = false;

    log::info!("Windows pipe client disconnected");
}