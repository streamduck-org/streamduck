use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread::spawn;
use named_pipe::{PipeOptions, PipeServer};
use streamduck_core::socket::{send_packet_as_is, SocketManager};
use streamduck_daemon::{WINDOWS_EVENT_PIPE_NAME, WINDOWS_REQUEST_PIPE_NAME};

pub fn open_socket(socket_manager: Arc<SocketManager>) {
    // Events socket
    {
        let socket = socket_manager.clone();
        spawn(move || open_event_socket(socket));
    }

    // Requests socket
    open_request_socket(socket_manager);
}

fn open_request_socket(socket_manager: Arc<SocketManager>) {
    loop {
        let instance = PipeOptions::new(WINDOWS_REQUEST_PIPE_NAME)
            .first(false)
            .single().expect("Failed to create named pipe server for requests");

        if let Ok(client) = instance.wait() {
            let manager = socket_manager.clone();
            spawn(move || handle_request_client(client, manager));
        }
    }
}

fn open_event_socket(socket_manager: Arc<SocketManager>) {
    loop {
        let instance = PipeOptions::new(WINDOWS_EVENT_PIPE_NAME)
            .first(false)
            .single().expect("Failed to create named pipe server for events");

        if let Ok(client) = instance.wait() {
            let manager = socket_manager.clone();
            spawn(move || handle_event_client(client, manager));
        }
    }
}

fn handle_request_client(client: PipeServer, manager: Arc<SocketManager>) {
    log::info!("Windows pipe request client connected");

    let mut stream = BufReader::new(client);

    let mut message = vec![];
    while let Ok(size) = stream.read_until(0x4, &mut message) {
        if size <= 0 {
            break;
        }

        if let Ok(message) = String::from_utf8(message.clone()) {
            log::info!("message {}", message.replace("\u{0004}", ""));
            match serde_json::from_str(&message.replace("\u{0004}", "")) {
                Ok(packet) => manager.received_message(stream.get_mut(), packet),
                Err(e) => log::warn!("Invalid message in sockets: {}", e)
            }
        }

        message.clear();
    }

    log::info!("Windows pipe request client disconnected");
}

fn handle_event_client(mut client: PipeServer, manager: Arc<SocketManager>) {
    log::info!("Windows pipe event client connected");

    let pool = manager.get_pool();

    loop {
        let message = pool.take_message();
        if send_packet_as_is(&mut client, message).is_err() {
            break;
        }
    }

    pool.close();

    log::info!("Windows pipe event client disconnected");
}