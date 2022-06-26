//! Socket related definitions

use std::io::Write;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::ops::Deref;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde::de::{DeserializeOwned, Error};
use serde_json::Value;
use tokio::sync::{Mutex, Notify, RwLock};
use async_recursion::async_recursion;
use crate::modules::events::SDGlobalEvent;

/// Type for listener's socket handles
pub type SocketHandle<'a> = &'a mut (dyn AsyncWrite + Unpin + Send);

/// Boxed socket listener
pub type UniqueSocketListener = Arc<dyn SocketListener + Send + Sync>;

/// Socket packet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketPacket {
    /// Data type
    pub ty: String,
    /// Possible requester, for letting client understand what response is for which request
    pub requester: Option<String>,
    /// Parse-able data
    pub data: Option<Value>
}

/// Socket listener, something that can listen in to socket connections
#[async_trait]
pub trait SocketListener {
    /// Called when message is received, handle can be used to send back a response
    async fn message(&self, socket: SocketHandle<'_>, packet: SocketPacket);
}

/// Trait for serialization and deserialization util functions
pub trait SocketData {
    const NAME: &'static str;
}

/// Attempts to parse socket data into a specified type
pub fn parse_packet_to_data<T: SocketData + DeserializeOwned>(packet: &SocketPacket) -> Result<T, serde_json::Error> {
    if packet.ty == T::NAME {
        if let Some(data) = &packet.data {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Err(serde_json::Error::custom("Missing data"))
        }
    } else {
        Err(serde_json::Error::custom("Invalid packet"))
    }
}

/// Checks if packet is of a certain type, for requests without any parameters
pub fn check_packet_for_data<T: SocketData>(packet: &SocketPacket) -> bool {
    packet.ty == T::NAME
}

/// Writes bytes in chunks
pub async fn write_in_chunks(handle: SocketHandle<'_>, data: String) -> Result<(), SocketError> {
    for chunk in data.into_bytes().chunks(250) {
        handle.write(chunk).await?;
    }

    Ok(())
}

/// Writes bytes in chunks with sync IO
pub fn write_in_chunks_sync(handle: &mut dyn Write, data: String) -> Result<(), SocketError> {
    for chunk in data.into_bytes().chunks(250) {
        handle.write(chunk)?;
    }

    Ok(())
}

/// Sends a packet with included requester ID from previous package
pub async fn send_packet<T: SocketData + Serialize>(handle: SocketHandle<'_>, previous_packet: &SocketPacket, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: previous_packet.requester.clone(),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is(handle, packet).await?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package with sync IO
pub async fn send_packet_sync<T: SocketData + Serialize>(handle: &mut dyn Write, previous_packet: &SocketPacket, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: previous_packet.requester.clone(),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is_sync(handle, packet)?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package
pub async fn send_packet_with_requester<T: SocketData + Serialize>(handle: SocketHandle<'_>, requester: &str, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is(handle, packet).await?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package with sync IO
pub fn send_packet_with_requester_sync<T: SocketData + Serialize>(handle: &mut dyn Write, requester: &str, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is_sync(handle, packet)?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package, without data
pub async fn send_no_data_packet_with_requester<T: SocketData>(handle: SocketHandle<'_>, requester: &str) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: None
    };

    send_packet_as_is(handle, packet).await?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package, without data, with sync IO
pub fn send_no_data_packet_with_requester_sync<T: SocketData>(handle: &mut dyn Write, requester: &str) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: None
    };

    send_packet_as_is_sync(handle, packet)?;

    Ok(())
}

/// Sends a packet as is
pub async fn send_packet_as_is(handle: SocketHandle<'_>, data: SocketPacket) -> Result<(), SocketError> {
    write_in_chunks(handle, format!("{}\u{0004}", serde_json::to_string(&data)?)).await?;

    Ok(())
}

/// Sends a packet as is with sync IO
pub fn send_packet_as_is_sync(handle: &mut dyn Write, data: SocketPacket) -> Result<(), SocketError> {
    write_in_chunks_sync(handle, format!("{}\u{0004}", serde_json::to_string(&data)?))?;

    Ok(())
}

/// Enumeration of various errors during sending and parsing packets
#[derive(Debug)]
pub enum SocketError {
    SerdeError(serde_json::Error),
    WriteError(std::io::Error),
}

impl From<serde_json::Error> for SocketError {
    fn from(err: serde_json::Error) -> Self {
        SocketError::SerdeError(err)
    }
}

impl From<std::io::Error> for SocketError {
    fn from(err: std::io::Error) -> Self {
        SocketError::WriteError(err)
    }
}

/// Manager of socket listeners
pub struct SocketManager {
    listeners: RwLock<Vec<UniqueSocketListener>>,
    pools: RwLock<Vec<Arc<SocketPool>>>
}

impl SocketManager {
    /// Creates a new socket manager
    pub fn new() -> Arc<SocketManager> {
        Arc::new(SocketManager {
            listeners: Default::default(),
            pools: Default::default()
        })
    }

    /// Adds socket listener to manager
    pub async fn add_listener(&self, listener: UniqueSocketListener) {
        self.listeners.write().await.push(listener);
    }

    /// Sends a message to all listeners, for socket implementation to trigger all listeners when message is received
    pub async fn received_message(&self, handle: SocketHandle<'_>, packet: SocketPacket) {
        for listener in self.listeners.read().await.deref() {
            listener.message(handle, packet.clone()).await;
        }
    }

    /// Creates a new message pool
    pub async fn get_pool(&self) -> Arc<SocketPool> {
        let mut pools = self.pools.write().await;

        let new_pool = Arc::new(SocketPool {
            messages: Mutex::new(vec![]),
            notification: Default::default(),
            is_open: RwLock::new(true)
        });

        pools.push(new_pool.clone());

        new_pool
    }

    /// For listeners or modules to send messages to all active socket connections, for event purposes
    pub async fn send_message(&self, packet: SocketPacket) {
        let mut pools = self.pools.write().await;

        let mut pools_to_delete = vec![];

        for (index, pool) in pools.iter().enumerate() {
            if *pool.is_open.read().await {
                pool.add_message(packet.clone()).await
            } else {
                pools_to_delete.push(index);
            }
        }

        for pool_to_delete in pools_to_delete {
            pools.remove(pool_to_delete);
        }
    }
}

pub async fn send_event_to_socket(socket_manager: &Arc<SocketManager>, event: SDGlobalEvent) {
    socket_manager.send_message(SocketPacket {
        ty: "event".to_string(),
        requester: None,
        data: Some(serde_json::to_value(event).unwrap())
    }).await
}

/// Pool of messages for socket implementations
pub struct SocketPool {
    messages: Mutex<Vec<SocketPacket>>,
    notification: Notify,
    is_open: RwLock<bool>
}

impl SocketPool {
    /// Puts message into the pool
    pub async fn add_message(&self, message: SocketPacket) {
        let mut messages = self.messages.lock().await;
        messages.insert(0, message);
        self.notification.notify_waiters();
    }

    /// Retrieves a message, will block if pool is currently empty
    #[async_recursion]
    pub async fn take_message(&self) -> SocketPacket {
        // Checking if message exists before waiting
        {
            let mut guard = self.messages.lock().await;
            if !guard.is_empty() {
                return guard.pop().unwrap();
            }
        }

        // Waiting for wake-up if empty pool
        self.notification.notified().await;
        let mut guard = self.messages.lock().await;

        if let Some(packet) = guard.pop() {
            packet
        } else {
            drop(guard);
            self.take_message().await
        }
    }

    pub async fn is_open(&self) -> bool {
        *self.is_open.read().await
    }

    pub async fn close(&self) {
        *self.is_open.write().await = false;
    }
}