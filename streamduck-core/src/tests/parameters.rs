use crate::parameters::{Parameter, ParameterVariant};

#[test]
fn test_parameters() {
    let my_param = Parameter::new_from_key(
        "test",
        "test",
        ParameterVariant::CollapsableMenu(vec![
            Parameter::new_from_key("header", "test.header", ParameterVariant::Header),
            Parameter::new_from_key(
                "label",
                "test.label",
                ParameterVariant::Label("what the".to_string()),
            ),
            Parameter::new_from_key(
                "array",
                "test.array",
                ParameterVariant::Array(vec![
                    vec![
                        Parameter::new_from_key("header", "test.header", ParameterVariant::Header),
                        Parameter::new_from_key(
                            "label",
                            "test.label",
                            ParameterVariant::Label("what the".to_string()),
                        ),
                    ],
                    vec![
                        Parameter::new_from_key("header", "test.header", ParameterVariant::Header),
                        Parameter::new_from_key(
                            "label",
                            "test.label",
                            ParameterVariant::Label("what the".to_string()),
                        ),
                    ],
                ]),
            ),
        ]),
    );

    println!("{}", serde_json::to_string_pretty(&my_param).unwrap());
}
