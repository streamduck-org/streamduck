mod response_handler;

use std::cell::RefCell;
use std::mem;
use std::net::SocketAddr;
use std::rc::Rc;
use futures::stream::SplitSink;
use futures::{select, FutureExt, SinkExt, StreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::{runtime, task};
use tokio::sync::{broadcast};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::LocalSet;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use response_handler::ResponseHandler;
use crate::config::CONFIG;
use crate::core::events::CoreEventMessage;
use crate::core::requests::{ClientRequestMessage, ClientResponseMessage};

#[derive(Debug)]
pub struct CoreChannels {
    pub ev_tx: UnboundedSender<CoreEventMessage>,
    pub rx: RefCell<UnboundedReceiver<ClientRequestMessage>>,
    pub tx: UnboundedSender<ClientResponseMessage>,
}

#[derive(Debug)]
pub struct ClientChannels {
    pub ev_rx: RefCell<UnboundedReceiver<CoreEventMessage>>,
    pub rx: RefCell<UnboundedReceiver<ClientResponseMessage>>,
    pub tx: UnboundedSender<ClientRequestMessage>,
}

pub fn get_channels() -> (CoreChannels, ClientChannels) {
    let (req_tx, req_rx) = unbounded_channel::<ClientRequestMessage>();
    let (res_tx, res_rx) = unbounded_channel::<ClientResponseMessage>();
    let (ev_tx, ev_rx) = unbounded_channel::<CoreEventMessage>();

    (
        CoreChannels {
            ev_tx,
            rx: req_rx.into(),
            tx: res_tx
        },
        ClientChannels {
            ev_rx: ev_rx.into(),
            rx: res_rx.into(),
            tx: req_tx
        }
    )
}

pub fn boot_client_thread(chan /* owo */: ClientChannels) {
    std::thread::spawn(move || {
        let rt = runtime::Builder::new_current_thread()
            .enable_io()
            .build().unwrap();

        // Transmutting chan into a static reference because tokio runtime will die
        // before this data is actually dropped. #trustmebro
        let chan_ref: &'static ClientChannels = unsafe { mem::transmute(&chan) };

        rt.block_on(async {
            let local_set = LocalSet::new();

            local_set.spawn_local(client_thread(chan_ref));

            local_set.await;
        });
    });
}

type EventReceiver = broadcast::Receiver<CoreEventMessage>;

async fn client_thread(chan: &'static ClientChannels) {
    let bind_address = CONFIG.load().client_srv_addr.clone();

    let socket = TcpListener::bind(&bind_address).await.expect("Failed to listen to address");
    info!("Listening on {}", bind_address);

    let response_handler = ResponseHandler::new(chan);
    let (ev_tx, _) = broadcast::channel::<CoreEventMessage>(10);

    let listen_ws = async |rx: ResponseHandler, ev: EventReceiver| {
        let (stream, addr) = socket.accept().await.unwrap();

        task::spawn_local(async move {
            let client = addr.to_string();

            match handle_connection(rx, ev, stream, addr).await {
                Ok(_) => {},
                Err(e) => error!("Error while communicating with '{}': {}", client, e)
            }
        });
    };

    loop {
        let mut ev_rx = chan.ev_rx.borrow_mut();

        select! {
            _ = listen_ws(response_handler.clone(), ev_tx.subscribe()).fuse() => (),
            response = response_handler.wait_for_responses().fuse() => {
                if let Err(e) = response {
                    error!("Response handler got error: {}. Exiting client thread!", e);
                    break;
                };
            },
            event = ev_rx.recv().fuse() => {
                let Some(event) = event else {
                    error!("Event channel is closed. Is core down? Exiting client thread!");
                    break;
                };

                ev_tx.send(event).ok();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WSError {
    pub error: String
}

type Outgoing = Rc<RefCell<SplitSink<WebSocketStream<TcpStream>, Message>>>;

async fn send_as_json<T: Serialize>(outgoing: Outgoing, addr: &SocketAddr, value: &T) -> anyhow::Result<()> {
    let str = serde_json::to_string(value)?;
    debug!("sending to '{}': {}", addr, str);
    outgoing.borrow_mut().send(Message::Text(Utf8Bytes::from(str))).await?;
    Ok(())
}

async fn handle_connection(rx: ResponseHandler, mut ev: EventReceiver, stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    info!("Client connected at '{}'", addr);

    let (outgoing, mut incoming) = ws_stream.split();
    let rc_outgoing = Rc::new(RefCell::new(outgoing));

    loop {
        select! {
            msg = incoming.next().fuse() => {
                let Some(received) = msg else {
                    break;
                };

                let msg = received?;

                if msg.is_ping() {
                    rc_outgoing.borrow_mut().send(Message::Pong(msg.into_data())).await?;
                    continue;
                }

                if !msg.is_text() {
                    continue
                }

                let addr_copy = addr.clone();
                let rx_copy = rx.clone();
                let rc_outgoing_copy = rc_outgoing.clone();
                task::spawn_local(async move {
                    match process_message(rx_copy, msg, &addr_copy).await {
                        Ok(res) => {
                            send_as_json(rc_outgoing_copy, &addr_copy, &res).await.unwrap()
                        }
                        Err(e) => {
                            error!("Failed to process message from '{}':\n{}", addr_copy, e);
                            send_as_json(rc_outgoing_copy, &addr_copy, &WSError {
                                error: e.to_string()
                            }).await.unwrap()
                        }
                    };
                });
            }
            event = ev.recv().fuse() => {
                let event = event?;
                send_as_json(rc_outgoing.clone(), &addr, &event).await?;
            }
        }
    }

    info!("Client connection closed at '{}'", addr);

    Ok(())
}

async fn process_message(rx: ResponseHandler, msg: Message, addr: &SocketAddr) -> anyhow::Result<ClientResponseMessage> {
    let text = msg.into_text()?.to_string();
    debug!("[{}]: {}", addr, text);
    let request = serde_json::from_str::<ClientRequestMessage>(&text)?;

    Ok(rx.request(request).await)
}