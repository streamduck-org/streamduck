use serde_json::json;
use streamduck_core::ui::{Field, FieldCondition, FieldType, UISchema, ValuePath};

fn main() {
    let schema: UISchema = vec![
        Field {
            value: Some(ValuePath::from("my_data")),
            title: Some("Some text here"),
            description: Some("Description here"),
            ty: FieldType::StringInput {
                disabled: false,
            },
            condition: FieldCondition::Not(
                Box::new(FieldCondition::Exists(
                    ValuePath::from("my_data")
                ))
            ),
        }
    ];

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
