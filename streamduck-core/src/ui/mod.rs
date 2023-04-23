/// Builders of UI types
pub mod builder;

use std::fmt::{Debug, Formatter};
use rmpv::Value;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Serialize as SerializeDerive, Deserialize as DeserializeDerive};

/// Base type used to describe field dialog of device config, action, overlay, rendering options, etc
pub type UISchema = Vec<Field>;

/// Description of a field on UI
#[derive(Clone, Debug, SerializeDerive, DeserializeDerive)]
pub struct Field {
    /// Value that the field is bound to
    pub value: Option<LodashValuePath>,

    /// Title of the field
    pub title: Option<&'static str>,

    /// Footer text of the field
    pub description: Option<&'static str>,

    /// Type of the field
    pub ty: FieldType,

    /// Condition on which the field will be shown in UI
    pub condition: FieldCondition
}

/// Different field types
#[derive(Clone, Debug, SerializeDerive, DeserializeDerive)]
pub enum FieldType {
    /// Displays title text in a large font
    Header,

    /// Displays description text in a normal text font with optional title
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

/// Condition that should be followed before the field will be visible
#[derive(Clone, Debug, SerializeDerive, DeserializeDerive)]
pub enum FieldCondition {
    /// Field will always be shown, always true
    Always,

    /// Field will never be shown, always false
    Never,

    /// Field will be shown if a certain path exists
    Exists(LodashValuePath),

    /// Field will be shown if the path has the same value as specified
    Equals(LodashValuePath, Value),

    /// Field will be shown if the path contains specified value, example "hello" in "hello world"
    Contains(LodashValuePath, Value),

    /// Field will be shown if the path's value matches the regex pattern, should use ECMAScript flavor of Regex
    RegexMatches {
        /// Path to a value to test
        value: LodashValuePath,

        /// Regex pattern to match
        pattern: String,

        /// Regex flags that should be used, example "gm"
        ///
        /// - g - Global search
        /// - i - Case-insensitive search
        /// - m - Allows `^` and `$` to match newline characters
        /// - s - Allows `.` to match newline characters
        /// - u - Unicode
        /// - y - Sticky search
        ///
        /// Look up "regex" in google if you need more help with this
        flags: String,
    },

    /// Field will be shown if value at the path will have value greater than specified
    GreaterThan(LodashValuePath, Value),

    /// Field will be shown if value at the path will have value equal or greater than specified
    GreaterThanOrEquals(LodashValuePath, Value),

    /// Field will be shown if value at the path will have value lesser than specified
    LesserThan(LodashValuePath, Value),

    /// Field will be shown if value at the path will have value equal or lesser than specified
    LesserThanOrEquals(LodashValuePath, Value),

    /// Field will be shown if the trigger inside is false
    Not(Box<FieldCondition>),

    /// Field will be shown if any trigger inside is true
    Or(Vec<FieldCondition>),

    /// Field will be shown if all conditions inside are true
    And(Vec<FieldCondition>)
}

/// Describes path to a value
#[derive(Clone, Debug)]
pub enum LodashValuePath {
    /// Lodash formatted path to a value, see [Lodash get](https://lodash.com/docs#get)
    Path(String),

    /// Path to a value formatted as an array, see array example at [Lodash get](https://lodash.com/docs#get)
    Breadcrumbs(Vec<String>)
}

impl From<String> for LodashValuePath {
    fn from(value: String) -> Self {
        LodashValuePath::Path(value)
    }
}

impl From<Vec<String>> for LodashValuePath {
    fn from(value: Vec<String>) -> Self {
        LodashValuePath::Breadcrumbs(value)
    }
}

impl From<&str> for LodashValuePath {
    fn from(value: &str) -> Self {
        LodashValuePath::Path(value.to_string())
    }
}

impl Serialize for LodashValuePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            LodashValuePath::Path(str) => {
                serializer.serialize_str(str)
            }
            LodashValuePath::Breadcrumbs(crumbs) => {
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
    type Value = LodashValuePath;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "string or array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        Ok(LodashValuePath::Path(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
        Ok(LodashValuePath::Path(v))
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

        Ok(LodashValuePath::Breadcrumbs(breadcrumbs))
    }
}

impl<'de> Deserialize<'de> for LodashValuePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(ValuePathVisitor)
    }
}