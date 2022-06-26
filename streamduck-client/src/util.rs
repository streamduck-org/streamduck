use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::DerefMut;
use std::sync::RwLockWriteGuard;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use streamduck_core::modules::components::ComponentDefinition;
use streamduck_core::modules::events::SDGlobalEvent;
use streamduck_core::socket::{parse_packet_to_data, send_no_data_packet_with_requester_sync, send_packet_with_requester_sync, SocketData, SocketPacket};
use crate::SDClientError;

/// Transforms module-component map into component map, if you don't care about module names for them
pub fn module_component_map_to_component_map(component_map: HashMap<String, HashMap<String, ComponentDefinition>>) -> HashMap<String, ComponentDefinition> {
    let mut map = HashMap::new();

    for (_, components) in component_map {
        map.extend(components)
    }

    map
}

pub fn read_socket(handle: &mut dyn BufRead) -> Result<SocketPacket, SDClientError> {
    let mut byte_array = vec![];
    handle.read_until(0x4, &mut byte_array)?;

    let line = String::from_utf8(byte_array)?;

    Ok(serde_json::from_str(line.replace("\u{0004}", "").trim())?)
}

pub fn read_response(handle: &mut dyn BufRead, requester: &str, mut event_buffer: Option<RwLockWriteGuard<Vec<SDGlobalEvent>>>) -> Result<SocketPacket, SDClientError> {
    loop {
        let packet = read_socket(handle)?;

        if packet.requester.as_ref().unwrap_or(&"".to_string()) == requester {
            return Ok(packet);
        } else {
            if let Some(buffer) = event_buffer.as_mut() {
                if let Some(data) = packet.data {
                    if let Ok(event) = serde_json::from_value(data) {
                        buffer.insert(0, event);
                    }
                }
            }
        }
    }
}

pub fn process_request<Req, Res, Han>(mut handle: &mut BufReader<Han>, request: &Req, event_buffer: Option<RwLockWriteGuard<Vec<SDGlobalEvent>>>) -> Result<Res, SDClientError>
    where
        Req: SocketData + Serialize,
        Res: SocketData + DeserializeOwned,
        Han: Read + Write
{
    let id = rand::thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect::<String>();

    send_packet_with_requester_sync(handle.get_mut(), &id, request)?;

    let packet = read_response(handle.deref_mut(), &id, event_buffer)?;

    Ok(parse_packet_to_data(&packet)?)
}

pub fn process_request_no_buffer<Req, Res, Han>(handle: &mut BufReader<Han>, request: &Req) -> Result<Res, SDClientError>
    where
        Req: SocketData + Serialize,
        Res: SocketData + DeserializeOwned,
        Han: Read + Write
{
    process_request(handle, request, None)
}

pub fn process_request_without_data<Res, Han>(mut handle: &mut BufReader<Han>, event_buffer: Option<RwLockWriteGuard<Vec<SDGlobalEvent>>>) -> Result<Res, SDClientError>
    where
        Res: SocketData + DeserializeOwned,
        Han: Read + Write
{
    let id = rand::thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect::<String>();

    send_no_data_packet_with_requester_sync::<Res>(handle.get_mut(), &id)?;

    let packet = read_response(handle.deref_mut(), &id, event_buffer)?;

    return Ok(parse_packet_to_data(&packet)?);
}

pub fn process_request_without_data_no_buffer<Res, Han>(handle: &mut BufReader<Han>) -> Result<Res, SDClientError>
    where
        Res: SocketData + DeserializeOwned,
        Han: Read + Write
{
    process_request_without_data(handle, None)
}

