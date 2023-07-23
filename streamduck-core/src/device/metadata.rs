use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::data::NamespacedName;
use crate::device::input::InputLayout;

/// Unique data that differentiates a certain device from any other
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct DeviceIdentifier {
    /// Driver that defined the device
    pub driver_name: NamespacedName,

    /// Identifier used for the device, eg. serial number
    pub identifier: String,

    /// Short description of the device, eg. "Elgato Stream Deck Plus"
    pub description: String,
}

impl DeviceIdentifier {
    /// Strips driver name from the identifier so it can be used in constructing other structs
    pub fn downgrade(self) -> PartialIdentifier {
        PartialIdentifier {
            identifier: self.identifier,
            description: self.description,
        }
    }
}

impl Display for DeviceIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "( driver: '{}', iden: '{}', desc: '{}' )", self.driver_name, self.identifier, self.description)
    }
}

/// Identifier retured by the implementations of the plugin, later transformed into [DeviceIdentifier] by streamduck-core
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PartialIdentifier {
    /// Identifier used for the device, eg. serial number
    pub identifier: String,

    /// Short description of the device, eg. "Elgato Stream Deck Plus"
    pub description: String,
}

impl PartialIdentifier {
    /// Upgrades partial identifier to actual identifier
    pub(crate) fn upgrade(self, driver_name: &NamespacedName) -> DeviceIdentifier {
        DeviceIdentifier {
            driver_name: driver_name.clone(),
            identifier: self.identifier,
            description: self.description,
        }
    }
}

/// Metadata describing the device
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeviceMetadata {
    /// Driver that found the device
    pub identifier: DeviceIdentifier,

    /// Input layout of the device
    pub layout: InputLayout,
}

/// Metadata returned by the implmentations of the plugin, later transformed into [DeviceMetadata] by streamduck-core
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PartialMetadata {
    /// Driver that found the device
    pub identifier: PartialIdentifier,

    /// Input layout of the device
    pub layout: InputLayout,
}

impl PartialMetadata {
    /// Upgrades partial metadata to actual metadata
    pub(crate) fn upgrade(self, driver_name: &NamespacedName) -> DeviceMetadata {
        DeviceMetadata {
            identifier: self.identifier.upgrade(driver_name),
            layout: self.layout,
        }
    }
}