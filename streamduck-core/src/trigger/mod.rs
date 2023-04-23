use rmpv::Value;
use serde::{Serialize, Deserialize};
use crate::data::Number;
use crate::util::traverse_msgpack;

/// Condition that needs to be met for action to be triggered
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriggerCondition {
    /// Name of the trigger trigger in form of "On press", "On touch", etc
    pub name: String,
    /// Condition of the trigger
    pub condition: Condition
}

/// Enum representation of the trigger
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Condition {
    /// Triggered if all conditions inside are true
    And(Vec<Condition>),

    /// Triggered if any condition inside is true
    Or(Vec<Condition>),

    /// Triggered if condition inside is not true
    Not(Box<Condition>),

    /// Checks if value exists at provided path. Note: every other condition already checks for existence if applicable
    Exists(ValuePath),

    /// Checks if value found at path is the same as provided one
    Equals(ValuePath, Value),

    /// Checks if number found at path is greater than provided one
    Greater(ValuePath, Number),

    /// Checks if number found at path is greater than or equals to provided one
    GreaterOrEquals(ValuePath, Number),

    /// Checks if number found at path is less than provided one
    Less(ValuePath, Number),

    /// Checks if number found at path is less than or equals to provided one
    LessOrEquals(ValuePath, Number),
}

impl Condition {
    /// Checks if the condition is true against provided value
    pub fn test(&self, data: &Value) -> bool {
        match self {
            Condition::Exists(path) => {
                traverse_msgpack(data, path.as_slice()).is_some()
            }

            Condition::Equals(path, expected_value) => {
                let Some(value) = traverse_msgpack(data, path.as_slice()) else {
                    return false;
                };

                value == expected_value
            }

            Condition::Greater(path, expected_value) => {
                let Some(value) = traverse_msgpack(data, path.as_slice()) else {
                    return false;
                };

                let Ok(number) = Number::try_from(value) else {
                    return false
                };

                number > *expected_value
            }

            Condition::GreaterOrEquals(path, expected_value) => {
                let Some(value) = traverse_msgpack(data, path.as_slice()) else {
                    return false;
                };

                let Ok(number) = Number::try_from(value) else {
                    return false
                };

                number >= *expected_value
            }
            Condition::Less(path, expected_value) => {
                let Some(value) = traverse_msgpack(data, path.as_slice()) else {
                    return false;
                };

                let Ok(number) = Number::try_from(value) else {
                    return false
                };

                number < *expected_value
            }
            Condition::LessOrEquals(path, expected_value) => {
                let Some(value) = traverse_msgpack(data, path.as_slice()) else {
                    return false;
                };

                let Ok(number) = Number::try_from(value) else {
                    return false
                };

                number <= *expected_value
            }

            Condition::And(conds) => {
                conds.iter().all(|x| x.test(data))
            }

            Condition::Or(conds) => {
                conds.iter().any(|x| x.test(data))
            }

            Condition::Not(cond) => {
                !cond.test(data)
            }
        }
    }
}

/// Array of keys/indexes to reach the value
pub type ValuePath = Vec<Value>;