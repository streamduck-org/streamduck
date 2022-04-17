//! Constants for defining current feature versions of the software
//!
//! This is used heavily with plugin compatibility checking, please report features your plugins use correctly.
//! If version of one of the listed features updates and your plugin is using it, it will be deemed incompatible.
//! This is made to ensure that the program will not crash because of API differences between plugin and the core.
//!
//! Versions here do not represent how up to date the features are, versions here are just for making sure plugins
//! are not using unsupported API.

/// API related to plugin definition and initialization, will be updated very rarely if ever
pub const PLUGIN_API: (&str, &str) = ("plugin_api", "0.1");
/// SDModule trait version, will be updated everytime there's a change to the module trait
pub const SDMODULE_TRAIT: (&str, &str) = ("sdmodule_trait", "0.1");
/// Core version, will be updated everytime there's change to core struct, probably never
pub const CORE: (&str, &str) = ("core", "0.1");
/// Core methods version, will be updated everytime there's changes to existing functions or functions get deleted
pub const CORE_METHODS: (&str, &str) = ("core_methods", "0.1");
/// Config, will be updated everytime there's changes to existing functions or functions get deleted
pub const CONFIG: (&str, &str) = ("config", "0.1");
/// Module manager, will be updated everytime there's changes to existing functions or functions get deleted
pub const MODULE_MANAGER: (&str, &str) = ("module_manager", "0.1");
/// Core events, will be updated everytime there's changes to existing events or an event was removed
pub const CORE_EVENTS: (&str, &str) = ("core_events", "0.1");
/// Socket API of daemon, mostly used for socket communication, will be updated everytime there's changes to existing requests or a request was removed
pub const SOCKET_API: (&str, &str) = ("socket_api", "0.1");
/// Rendering version, will be updated everytime there's changes to existing rendering API for plugins
pub const RENDERING: (&str, &str) = ("rendering", "0.1");

/// Constant array of currently supported features, can also be used for plugin to specify using all of the features
pub const SUPPORTED_FEATURES: &[(&str, &str)] = &[
    PLUGIN_API,
    SDMODULE_TRAIT,
    CORE,
    CORE_METHODS,
    CONFIG,
    MODULE_MANAGER,
    CORE_EVENTS,
    RENDERING,
    SOCKET_API
];