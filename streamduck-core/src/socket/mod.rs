//! Socket related definitions

use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use serde::{Deserialize, Serialize};
use serde::de::{DeserializeOwned, Error};
use serde_json::Value;
use crate::modules::events::SDGlobalEvent;

/// Type for listener's socket handles
pub type SocketHandle<'a> = &'a mut dyn Write;

/// Boxed socket listener
pub type BoxedSocketListener = Box<dyn SocketListener + Send + Sync>;

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
pub trait SocketListener {
    /// Called when message is received, handle can be used to send back a response
    fn message(&self, socket: SocketHandle, packet: SocketPacket);
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
pub fn write_in_chunks(handle: SocketHandle, data: String) -> Result<(), SocketError> {
    for chunk in data.into_bytes().chunks(250) {
        handle.write(chunk)?;
    }

    Ok(())
}

/// Sends a packet with included requester ID from previous package
pub fn send_packet<T: SocketData + Serialize>(handle: SocketHandle, previous_packet: &SocketPacket, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: previous_packet.requester.clone(),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is(handle, packet)?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package
pub fn send_packet_with_requester<T: SocketData + Serialize>(handle: SocketHandle, requester: &str, data: &T) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: Some(serde_json::to_value(data)?)
    };

    send_packet_as_is(handle, packet)?;

    Ok(())
}

/// Sends a packet with included requester ID from previous package, without data
pub fn send_no_data_packet_with_requester<T: SocketData>(handle: SocketHandle, requester: &str) -> Result<(), SocketError> {
    let packet = SocketPacket {
        ty: T::NAME.to_string(),
        requester: Some(requester.to_string()),
        data: None
    };

    send_packet_as_is(handle, packet)?;

    Ok(())
}

/// Sends a packet as is
pub fn send_packet_as_is(handle: SocketHandle, data: SocketPacket) -> Result<(), SocketError> {
    write_in_chunks(handle, format!("{}\u{0004}", serde_json::to_string(&data)?))?;

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
    listeners: RwLock<Vec<BoxedSocketListener>>,
    pools: RwLock<Vec<Arc<RwLock<SocketPool>>>>
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
    pub fn add_listener(&self, listener: BoxedSocketListener) {
        self.listeners.write().unwrap().push(listener);
    }

    /// Sends a message to all listeners, for socket implementation to trigger all listeners when message is received
    pub fn received_message(&self, handle: SocketHandle, packet: SocketPacket) {
        for listener in self.listeners.read().unwrap().deref() {
            listener.message(handle, packet.clone());
        }
    }

    /// Creates a new message pool
    pub fn get_pool(&self) -> Arc<RwLock<SocketPool>> {
        let mut pools = self.pools.write().unwrap();

        let new_pool = Arc::new(RwLock::new(SocketPool {
            messages: vec![],
            is_open: true
        }));

        pools.push(new_pool.clone());

        new_pool
    }

    /// For listeners or modules to send messages to all active socket connections, for event purposes
    pub fn send_message(&self, packet: SocketPacket) {
        let mut pools = self.pools.write().unwrap();

        pools.retain(|x| {
            if let Ok(x) = x.read() {
                x.is_open()
            } else {
                false
            }
        });

        for pool in pools.iter() {
            if let Ok(mut pool) = pool.write() {
                pool.add_message(packet.clone())
            }
        }
    }
}

/// Sends packet to all socket connections in different thread, so current thread won't have to wait for write locks
pub fn send_socket_message(socket_manager: &Arc<SocketManager>, packet: SocketPacket) {
    let socket_manager = socket_manager.clone();
    spawn(move || {
        socket_manager.send_message(packet);
    });
}

pub fn send_event_to_socket(socket_manager: &Arc<SocketManager>, event: SDGlobalEvent) {
    send_socket_message(socket_manager, SocketPacket {
        ty: "event".to_string(),
        requester: None,
        data: Some(serde_json::to_value(event).unwrap())
    })
}

/// Pool of messages for socket implementations
pub struct SocketPool {
    messages: Vec<SocketPacket>,
    is_open: bool
}

impl SocketPool {
    pub fn add_message(&mut self, message: SocketPacket) {
        self.messages.insert(0, message);
    }

    pub fn take_message(&mut self) -> Option<SocketPacket> {
        self.messages.pop()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}