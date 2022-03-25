//! Requests related to modules
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use streamduck_core::modules::components::{ComponentDefinition, UIPathValue};
use streamduck_core::modules::{add_element_module_setting, PluginMetadata, remove_element_module_setting, set_module_setting};
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::convert_value_to_path;
use crate::daemon_data::{DaemonListener, DaemonRequest};

/// Request for getting all loaded modules
#[derive(Serialize, Deserialize)]
pub struct ListModules {
    pub modules: Vec<PluginMetadata>
}

impl SocketData for ListModules {
    const NAME: &'static str = "list_modules";
}

impl DaemonRequest for ListModules {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListModules>(&packet) {
            let modules = listener.module_manager.get_module_list()
                .iter()
                .map(|m| m.metadata())
                .collect::<Vec<PluginMetadata>>();

            send_packet(handle, &packet, &ListModules {
                modules
            }).ok();
        }
    }
}

/// Request for getting all components defined by all modules
#[derive(Serialize, Deserialize)]
pub struct ListComponents {
    /// Hashmap of module name to component map
    pub components: HashMap<String, HashMap<String, ComponentDefinition>>
}

impl SocketData for ListComponents {
    const NAME: &'static str = "list_components";
}

impl DaemonRequest for ListComponents {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ListComponents>(&packet) {
            let components = listener.module_manager.get_module_component_map()
                .into_iter()
                .map(|(n, c)| (n, c.into_iter().collect()))
                .collect();

            send_packet(handle, packet, &ListComponents {
                components
            }).ok();
        }
    }
}

/// Request for getting module settings
#[derive(Serialize, Deserialize)]
pub struct GetModuleValues {
    pub module_name: String,
}

/// Response of [GetModuleValues] request
#[derive(Serialize, Deserialize)]
pub enum GetModuleValuesResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module values were successfully retrieved
    Values(Vec<UIPathValue>),
}

impl SocketData for GetModuleValues {
    const NAME: &'static str = "get_module_values";
}

impl SocketData for GetModuleValuesResult {
    const NAME: &'static str = "get_module_values";
}

impl DaemonRequest for GetModuleValues {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetModuleValues>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    let values = module.settings()
                        .into_iter()
                        .map(|x| convert_value_to_path(x, ""))
                        .collect();

                    send_packet(handle, packet, &GetModuleValuesResult::Values(values)).ok();
                    return;
                }
            }

            send_packet(handle, packet, &GetModuleValuesResult::ModuleNotFound).ok();
        }
    }
}

/// Request for adding element into array of module's setting
#[derive(Serialize, Deserialize)]
pub struct AddModuleValue {
    pub module_name: String,
    pub path: String
}

/// Response of [AddModuleValue] request
#[derive(Serialize, Deserialize)]
pub enum AddModuleValueResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module value failed to be added
    FailedToAdd,

    /// Sent if module value was successfully added
    Added
}

impl SocketData for AddModuleValue {
    const NAME: &'static str = "add_module_value";
}

impl SocketData for AddModuleValueResult {
    const NAME: &'static str = "add_module_value";
}

impl DaemonRequest for AddModuleValue {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddModuleValue>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    if add_element_module_setting(&module, &request.path) {
                        send_packet(handle, packet, &AddModuleValueResult::Added).ok();
                    } else {
                        send_packet(handle, packet, &AddModuleValueResult::FailedToAdd).ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &AddModuleValueResult::ModuleNotFound).ok();
        }
    }
}

/// Request for removing element from array of module's setting
#[derive(Serialize, Deserialize)]
pub struct RemoveModuleValue {
    pub module_name: String,
    pub path: String,
    pub index: usize
}

/// Response of [RemoveModuleValue] request
#[derive(Serialize, Deserialize)]
pub enum RemoveModuleValueResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module value failed to be removed
    FailedToRemove,

    /// Sent if module value was successfully removed
    Removed
}

impl SocketData for RemoveModuleValue {
    const NAME: &'static str = "remove_module_value";
}

impl SocketData for RemoveModuleValueResult {
    const NAME: &'static str = "remove_module_value";
}

impl DaemonRequest for RemoveModuleValue {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveModuleValue>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    if remove_element_module_setting(&module, &request.path, request.index) {
                        send_packet(handle, packet, &RemoveModuleValueResult::Removed).ok();
                    } else {
                        send_packet(handle, packet, &RemoveModuleValueResult::FailedToRemove).ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &RemoveModuleValueResult::ModuleNotFound).ok();
        }
    }
}

/// Request for setting a value to module's setting
#[derive(Serialize, Deserialize)]
pub struct SetModuleValue {
    pub module_name: String,
    pub value: UIPathValue
}

/// Response of [SetModuleValue] request
#[derive(Serialize, Deserialize)]
pub enum SetModuleValueResult {
    /// Sent if module wasn't found
    ModuleNotFound,

    /// Sent if module value failed to be set
    FailedToSet,

    /// Sent if module value was successfully set
    Set
}

impl SocketData for SetModuleValue {
    const NAME: &'static str = "set_module_value";
}

impl SocketData for SetModuleValueResult {
    const NAME: &'static str = "set_module_value";
}

impl DaemonRequest for SetModuleValue {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetModuleValue>(packet) {
            for module in listener.module_manager.get_module_list() {
                if module.name() == request.module_name {
                    if set_module_setting(&module, request.value) {
                        send_packet(handle, packet, &SetModuleValueResult::Set).ok();
                    } else {
                        send_packet(handle, packet, &SetModuleValueResult::FailedToSet).ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &SetModuleValueResult::ModuleNotFound).ok();
        }
    }
}