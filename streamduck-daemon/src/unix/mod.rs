use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::{fs, thread};
use std::sync::Arc;
use streamduck_daemon::socket::SocketManager;

const SOCKET_PATH: &'static str = "/tmp/streamduck.sock";

pub fn remove_socket() {
    fs::remove_file(SOCKET_PATH).ok();
}

pub fn open_socket(socket_manager: Arc<SocketManager>) {
    remove_socket();
    let listener = UnixListener::bind("/tmp/streamduck.sock").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let manager = socket_manager.clone();
                thread::spawn(move || handle_client(stream, manager));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(stream: UnixStream, socket_manager: Arc<SocketManager>) {
    println!("Unix Socket client connected");
    let mut stream = BufReader::new(stream);

    let mut message = String::new();
    while let Ok(size) = stream.read_line(&mut message) {
        if size <= 0 {
            break;
        }

        match serde_json::from_str(&message) {
            Ok(packet) => socket_manager.message(stream.get_mut(), packet),
            Err(e) => log::warn!("Invalid message in sockets: {}", e)
        }

        message.clear();
    }
    println!("Unix Socket client disconnected");
}