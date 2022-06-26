use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::select;
use streamduck_core::socket::{send_packet_as_is, SocketManager};
use streamduck_daemon::WINDOWS_PIPE_NAME;

pub async fn open_socket(socket_manager: Arc<SocketManager>) {
    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(WINDOWS_PIPE_NAME).unwrap();

    loop {
        if let Ok(_) = server.connect().await {
            let man = socket_manager.clone();
            tokio::spawn(async move { handle_client(server, man).await });
        }

        server = ServerOptions::new()
            .create(WINDOWS_PIPE_NAME).unwrap()
    }
}

async fn handle_client(client: NamedPipeServer, manager: Arc<SocketManager>) {
    log::info!("Windows pipe request client connected");

    let mut stream = BufStream::new(client);
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

    log::info!("Windows pipe request client disconnected");
}