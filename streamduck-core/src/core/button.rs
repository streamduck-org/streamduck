use std::collections::HashMap;
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json::{Error, Value};
use crate::core::UniqueButton;

/// Button definition, it's simply a hashmap, but is used to represent all the components of the button
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Button(pub HashMap<String, Value>);

impl Button {
    /// Creates a new empty button
    pub fn new() -> Button {
        Button::default()
    }

    /// Inserts new component into the button
    pub fn insert_component<T: Component + Serialize>(&mut self, component: T) -> Result<Option<Value>, ParseError> {
        Ok(self.0.insert(T::NAME.to_string(), serialize_component(component)?))
    }

    /// Removes component of provided type from the button
    pub fn remove_component<T: Component>(&mut self) -> Option<Value> {
        self.0.remove(T::NAME)
    }

    /// Returns list of components in the button
    pub fn component_names(&self) -> Vec<String> {
        self.0.keys().map(|x| x.clone()).collect()
    }

    /// Checks if button contains specified component name
    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
}

/// Component trait, simply provides name for component generic methods
pub trait Component {
    const NAME: &'static str;
}

/// Attempts to retrieve a component from a button
pub fn parse_button_to_component<T: Component + DeserializeOwned>(button: &Button) -> Result<T, ParseError> {
    if let Some(component_value) = button.0.get(T::NAME) {
        Ok(serde_json::from_value(component_value.clone())?)
    } else {
        Err(ParseError::Missing)
    }
}

/// Attempts to retrieve a component from reference counted button
pub async fn parse_unique_button_to_component<T: Component + DeserializeOwned>(button: &UniqueButton) -> Result<T, ParseError> {
    parse_button_to_component(button.read().await.deref())
}

/// Serializes component into JSON
pub fn serialize_component<T: Component + Serialize>(component: T) -> Result<Value, ParseError> {
    Ok(serde_json::to_value(component)?)
}

/// Parse error used for functions in this module
#[derive(Debug)]
pub enum ParseError {
    Missing,
    JSONError(serde_json::Error)
}

impl From<serde_json::Error> for ParseError {
    fn from(err: Error) -> Self {
        ParseError::JSONError(err)
    }
}