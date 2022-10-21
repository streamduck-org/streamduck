use std::fs;
use std::sync::Arc;
use tokio::io::BufStream;
use tokio::net::{UnixListener, UnixStream};
use tokio::select;
use tokio::io::AsyncBufReadExt;
use streamduck_core::socket::{send_packet_as_is, SocketManager};
use streamduck_daemon::UNIX_SOCKET_PATH;

pub fn remove_socket() {
    fs::remove_file(UNIX_SOCKET_PATH).ok();
}

pub async fn open_socket(socket_manager: Arc<SocketManager>) {
    remove_socket();
    let listener = UnixListener::bind(UNIX_SOCKET_PATH).unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let man = socket_manager.clone();
                tokio::spawn(async move { handle_client(stream, man).await });
            }
            Err(err) => {
                log::error!("Unix socket error: {}", err);
                break;
            }
        }
    }
}

async fn handle_client(stream: UnixStream, manager: Arc<SocketManager>) {
    log::info!("Unix Socket client connected");

    let mut stream=  BufStream::new(stream);
    let pool = manager.get_pool().await;


    loop {
        let mut message = vec![];
        select! {
            // Send event to socket if event is received
            message = pool.take_message() => {
                if send_packet_as_is(stream.get_mut(), message).await.is_err() {
                    break;
                }
            }

            // Process socket request if request is received
            size_result = stream.read_until(0x4, &mut message) => {
                if let Ok(size) = size_result {
                    if size <= 0 {
                        break;
                    }

                    if let Ok(message) = String::from_utf8(message.clone()) {
                        match serde_json::from_str(&message.replace("\u{0004}", "")) {
                            Ok(packet) => {
                                manager.received_message(stream.get_mut(), packet).await;
                            }

                            Err(e) => log::warn!("Invalid message in sockets: {}", e)
                        }
                    }

                    message.clear();
                }
            }
        }
    }

    log::info!("Unix Socket client disconnected");
}