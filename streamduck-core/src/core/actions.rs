use std::error::Error;
use std::sync::Weak;
use async_trait::async_trait;
use rmpv::Value;
use serde::{Deserialize, Serialize};
use crate::data::{Source, NamespacedName};
use crate::event::Event;
use crate::plugin::Plugin;
use crate::ui::UISchema;

/// Action that can perform things
pub struct Action {
    /// Plugin that the action originated from
    pub original_plugin: Weak<Plugin>,

    /// Name of the action
    pub name: String,

    /// Implementation of the action
    pub implement: Box<dyn ActionImpl>,

    /// UI Schema of the action
    pub ui: UISchema
}

impl Action {
    /// Namespaced name of the action
    pub fn namespaced_name(&self) -> Option<NamespacedName> {
        let plugin = self.original_plugin.upgrade()?;

        Some(NamespacedName::from_plugin(&plugin, &self.name))
    }
}

/// Action implementation
#[async_trait]
pub trait ActionImpl {
    /// Called when action options on some button got changed. Updated options are given along with new data separately
    async fn options_changed(&self, source: &Source, options: &Value, new_data: Value);

    /// Invokes the action with the source where the invokation originated, event that triggered the action and options of the action
    async fn invoke(&self, source: &Source, cause: &Event, options: &Value) -> ActionResult;
}

/// Result of action invokation
#[derive(Debug)]
pub enum ActionResult {
    /// Action was executed successfully
    Success,

    /// Action was executed successfully, and the success should be reported to the user as an icon
    VisibleSuccess,

    /// Action failed to execute, failure gets reported to the user as an icon
    Failure,

    /// Error occured while trying to execute the action
    Error(Box<dyn Error>)
}

/// Data to be used by the actions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionData {
    /// Name of the action responsible for this
    pub name: NamespacedName,

    /// Options for the action
    pub options: Value
}