//! Requests related to modules
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use streamduck_core::modules::components::{ComponentDefinition, UIPathValue};
use streamduck_core::modules::{add_element_module_setting, PluginMetadata, remove_element_module_setting, set_module_setting};
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::convert_value_to_path;
use crate::daemon_data::{DaemonListener, DaemonRequest};
use streamduck_core::async_trait;

/// Request for getting all loaded modules
#[derive(Serialize, Deserialize)]
pub struct ListModules {
    pub modules: Vec<PluginMetadata>
}

impl SocketData for ListModules {
    const NAME: &'static str = "list_modules";
}

#[async_trait]
impl DaemonRequest for ListModules {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<ListModules>(&packet) {
            let modules = listener.module_manager.get_module_list().await
                .iter()
                .map(|m| m.metadata())
                .collect::<Vec<PluginMetadata>>();

            send_packet(handle, &packet, &ListModules {
                modules
            }).await.ok();
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

#[async_trait]
impl DaemonRequest for ListComponents {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<ListComponents>(&packet) {
            let components = listener.module_manager.get_module_component_map().await
                .into_iter()
                .map(|(n, c)| (n, c.into_iter().collect()))
                .collect();

            send_packet(handle, packet, &ListComponents {
                components
            }).await.ok();
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

#[async_trait]
impl DaemonRequest for GetModuleValues {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetModuleValues>(packet) {
            for module in listener.module_manager.get_module_list().await {
                if module.name() == request.module_name {
                    let values = module.settings(listener.core_manager.clone()).await
                        .into_iter()
                        .map(|x| convert_value_to_path(x, ""))
                        .collect();

                    send_packet(handle, packet, &GetModuleValuesResult::Values(values)).await.ok();
                    return;
                }
            }

            send_packet(handle, packet, &GetModuleValuesResult::ModuleNotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for AddModuleValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddModuleValue>(packet) {
            for module in listener.module_manager.get_module_list().await {
                if module.name() == request.module_name {
                    if add_element_module_setting(listener.core_manager.clone(), &module, &request.path).await {
                        send_packet(handle, packet, &AddModuleValueResult::Added).await.ok();
                    } else {
                        send_packet(handle, packet, &AddModuleValueResult::FailedToAdd).await.ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &AddModuleValueResult::ModuleNotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for RemoveModuleValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveModuleValue>(packet) {
            for module in listener.module_manager.get_module_list().await {
                if module.name() == request.module_name {
                    if remove_element_module_setting(listener.core_manager.clone(), &module, &request.path, request.index).await {
                        send_packet(handle, packet, &RemoveModuleValueResult::Removed).await.ok();
                    } else {
                        send_packet(handle, packet, &RemoveModuleValueResult::FailedToRemove).await.ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &RemoveModuleValueResult::ModuleNotFound).await.ok();
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

#[async_trait]
impl DaemonRequest for SetModuleValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetModuleValue>(packet) {
            for module in listener.module_manager.get_module_list().await {
                if module.name() == request.module_name {
                    if set_module_setting(listener.core_manager.clone(), &module, request.value).await {
                        send_packet(handle, packet, &SetModuleValueResult::Set).await.ok();
                    } else {
                        send_packet(handle, packet, &SetModuleValueResult::FailedToSet).await.ok();
                    }

                    return;
                }
            }

            send_packet(handle, packet, &SetModuleValueResult::ModuleNotFound).await.ok();
        }
    }
}