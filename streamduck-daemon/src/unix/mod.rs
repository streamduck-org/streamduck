use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::{fs, thread};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use streamduck_core::socket::{send_packet_as_is, SocketManager};
use streamduck_daemon::UNIX_SOCKET_PATH;

pub fn remove_socket() {
    fs::remove_file(UNIX_SOCKET_PATH).ok();
}

pub fn open_socket(socket_manager: Arc<SocketManager>) {
    remove_socket();
    let listener = UnixListener::bind(UNIX_SOCKET_PATH).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let manager = socket_manager.clone();
                thread::spawn(move || handle_client(stream, manager));
            }
            Err(err) => {
                log::error!("Unix socket error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(stream: UnixStream, socket_manager: Arc<SocketManager>) {
    log::info!("Unix Socket client connected");

    if let Ok(write_stream) = stream.try_clone() {
        let pool = socket_manager.get_pool();

        // Write thread
        thread::spawn(move || {
            let mut stream = write_stream;
            let pool = pool;

            loop {
                let message = pool.take_message();
                if send_packet_as_is(&mut stream, message).is_err() {
                    break;
                }
            }

            pool.close();
        });

        // Read thread
        let mut stream = BufReader::new(stream);

        let mut message = vec![];
        while let Ok(size) = stream.read_until(0x4, &mut message) {
            if size <= 0 {
                break;
            }

            if let Ok(message) = String::from_utf8(message.clone()) {
                match serde_json::from_str(&message.replace("\u{0004}", "")) {
                    Ok(packet) => socket_manager.received_message(stream.get_mut(), packet),
                    Err(e) => log::warn!("Invalid message in sockets: {}", e)
                }
            }

            message.clear();
        }
    }

    log::info!("Unix Socket client disconnected");
}