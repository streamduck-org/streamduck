use base64::Engine;
use rmpv::Value;
use streamduck_core::data::Number;
use streamduck_core::{msgslice, msgvec};
use streamduck_core::trigger::{Condition, TriggerCondition};
use streamduck_core::ui::{Field, FieldCondition, FieldType, UISchema, LodashValuePath};
use streamduck_core::util::{traverse_msgpack, traverse_msgpack_mut};

fn main() {
    let schema: UISchema = vec![
        Field {
            value: Some(LodashValuePath::from("my_data")),
            title: Some("Some text here"),
            description: Some("Description here"),
            ty: FieldType::StringInput {
                disabled: false,
            },
            condition: FieldCondition::Not(
                Box::new(FieldCondition::Exists(
                    LodashValuePath::from("my_data")
                ))
            ),
        }
    ];

    let byte_array = rmp_serde::to_vec_named(&schema).unwrap();

    let mut enm: Value = rmp_serde::from_slice(&byte_array).unwrap();

    let condition = Condition::Equals(
        msgvec!(0, "title"),
        "Some text here".into()
    );

    println!("{:?}", condition.test(&enm));
}
