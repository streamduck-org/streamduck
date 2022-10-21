use streamduck_core::modules::components::{UIFieldType, UIFieldValue};

pub fn print_table(table: Vec<Vec<&str>>, first_separator: &str, separator: &str) {
    let mut max_len = vec![];

    // Calculating max size for each column
    for column in &table {
        let mut len = 0;

        for item in column {
            len = len.max(item.len());
        }

        max_len.push(len);
    }

    // Printing table
    if table.len() > 0 {
        for y in 0..table[0].len() {
            let separator = if y == 0 {
                first_separator
            } else {
                separator
            };

            for x in 0..table.len() {
                if y == 0 {
                    print!("{} {: <w$} ", separator, table[x][y], w = max_len[x])
                } else {
                    print!("{} {: >w$} ", separator, table[x][y], w = max_len[x])
                }
            }

            println!("{}", separator);
        }
    }
}

pub fn print_table_with_strings(table: Vec<Vec<String>>, first_separator: &str, separator: &str) {
    print_table(
        table.iter()
            .map(|v| {
                v.iter().map(|s| s.as_str()).collect()
            })
            .collect(),
        first_separator,
        separator
    );
}

pub fn parse_string_to_value<T>(value: &str, ty: &UIFieldType) -> Option<UIFieldValue<T>> {
    match ty {
        UIFieldType::Header |
        UIFieldType::Label |
        UIFieldType::ImagePreview => None,

        UIFieldType::InputFieldFloat => {
            if let Ok(value) = value.parse::<f32>() {
                Some(UIFieldValue::InputFieldFloat(value))
            } else {
                None
            }
        }

        UIFieldType::InputFieldInteger => {
            if let Ok(value) = value.parse::<i32>() {
                Some(UIFieldValue::InputFieldInteger(value))
            } else {
                None
            }
        }

        UIFieldType::InputFieldString => {
            Some(UIFieldValue::InputFieldString(value.to_string()))
        }

        UIFieldType::InputFieldFloat2 => {
            let mut parts = value.split(",");

            if let Ok(f1) = (parts.next().unwrap_or_default()).parse::<f32>() {
                if let Ok(f2) = (parts.next().unwrap_or_default()).parse::<f32>() {
                    Some(UIFieldValue::InputFieldFloat2(f1, f2))
                } else {
                    None
                }
            } else {
                None
            }
        }

        UIFieldType::InputFieldInteger2 => {
            let mut parts = value.split(",");

            if let Ok(i1) = (parts.next().unwrap_or_default()).parse::<i32>() {
                if let Ok(i2) = (parts.next().unwrap_or_default()).parse::<i32>() {
                    Some(UIFieldValue::InputFieldInteger2(i1, i2))
                } else {
                    None
                }
            } else {
                None
            }
        }

        UIFieldType::InputFieldUnsignedInteger => {
            if let Ok(value) = value.parse::<u32>() {
                Some(UIFieldValue::InputFieldUnsignedInteger(value))
            } else {
                None
            }
        }

        UIFieldType::ValueSliderFloat(_) => {
            if let Ok(value) = value.parse::<f32>() {
                Some(UIFieldValue::InputFieldFloat(value))
            } else {
                None
            }
        }

        UIFieldType::ValueSliderInteger(_) => {
            if let Ok(value) = value.parse::<i32>() {
                Some(UIFieldValue::ValueSliderInteger(value))
            } else {
                None
            }
        }

        UIFieldType::Collapsable => None,
        UIFieldType::Array(_) => None,

        UIFieldType::Choice(_) => {
            Some(UIFieldValue::InputFieldString(value.to_string()))
        }

        UIFieldType::Checkbox { .. } => {
            if let Ok(value) = value.parse::<bool>() {
                Some(UIFieldValue::Checkbox(value))
            } else {
                None
            }
        }

        UIFieldType::Color => {
            let mut parts = value.split(",");

            if let Ok(c1) = (parts.next().unwrap_or_default()).parse::<u8>() {
                if let Ok(c2) = (parts.next().unwrap_or_default()).parse::<u8>() {
                    if let Ok(c3) = (parts.next().unwrap_or_default()).parse::<u8>() {
                        if let Ok(c4) = (parts.next().unwrap_or_default()).parse::<u8>() {
                            Some(UIFieldValue::Color(c1, c2, c3, c4))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }

        UIFieldType::ImageData => {
            Some(UIFieldValue::ImageData(value.to_string()))
        }

        UIFieldType::ExistingImage => {
            Some(UIFieldValue::ExistingImage(value.to_string()))
        }

        UIFieldType::Font => {
            Some(UIFieldValue::Font(value.to_string()))
        }

        UIFieldType::Button { disabled } => {
            if *disabled {
                None
            } else {
                Some(UIFieldValue::Button)
            }
        }
    }
}