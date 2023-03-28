use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Serialize as SerializeDerive, Deserialize as DeserializeDerive};
use serde_json::Value;

/// Base type used to describe field dialog of device config, action, overlay, rendering options, etc
pub type UISchema = Vec<Field>;

/// Description of a field on UI
#[derive(Clone, Debug, SerializeDerive, DeserializeDerive)]
pub struct Field {
    /// Value that the field is bound to
    pub value: Option<ValuePath>,

    /// Title of the field
    pub title: Option<&'static str>,

    /// Footer text of the field
    pub description: Option<&'static str>,

    /// Type of the field
    pub ty: FieldType
}

/// Different field types
#[derive(Clone, Debug, SerializeDerive, DeserializeDerive)]
pub enum FieldType {
    /// Displays title text in a large font
    Header,

    /// Displays title text in a normal text font
    StaticText,

    /// Displays preset text or value if bound
    Label {
        /// Text to display if there's no bound value
        text: Option<&'static str>
    },

    /// Field that can contain any UTF-8 character
    StringInput {
        /// If the field should appear disabled
        disabled: bool
    },

    /// Field that can contain any whole number
    IntegerInput {
        /// If the field should appear disabled
        disabled: bool
    },

    /// Field that can contain any real number
    NumberInput {
        /// If the field should appear disabled
        disabled: bool
    },

    /// Field that contains elements of an array
    Array {
        /// Fields that should be displayed for each element
        element: UISchema,

        /// Data template of a new element, if not set, UI cannot add elements to the array
        new_element_template: Option<Value>,

        /// If UI should be able to remove elements from the array
        allow_removing: bool,

        /// If UI should be able to reorder elements in the array
        allow_reorder: bool
    },

    /// Field that contains nested fields
    NestedFields {
        /// Nested fields
        fields: UISchema,

        /// If the nested fields should be inside of a collapsable menu
        collapsable: bool
    }
}

/// Describes path to a value
#[derive(Clone, Debug)]
pub enum ValuePath {
    /// Lodash formatted path to a value, see [Lodash get](https://lodash.com/docs#get)
    Path(String),

    /// Path to a value formatted as an array, see array example at [Lodash get](https://lodash.com/docs#get)
    Breadcrumbs(Vec<String>)
}

impl Serialize for ValuePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            ValuePath::Path(str) => {
                serializer.serialize_str(str)
            }
            ValuePath::Breadcrumbs(crumbs) => {
                let mut seq = serializer.serialize_seq(Some(crumbs.len()))?;

                for crumb in crumbs {
                    seq.serialize_element(crumb)?;
                }

                seq.end()
            }
        }
    }
}

struct ValuePathVisitor;

impl<'de> Visitor<'de> for ValuePathVisitor {
    type Value = ValuePath;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "string or array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        Ok(ValuePath::Path(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
        Ok(ValuePath::Path(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut breadcrumbs: Vec<String> = if let Some(hint) = seq.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };

        while let Some(value) = seq.next_element::<String>()? {
            breadcrumbs.push(value);
        }

        Ok(ValuePath::Breadcrumbs(breadcrumbs))
    }
}

impl<'de> Deserialize<'de> for ValuePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(ValuePathVisitor)
    }
}