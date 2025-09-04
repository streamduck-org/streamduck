use itertools::Itertools;
use serde_json::Value;
use crate::api::value_type::ValueType;

pub fn validate_json_value(value: &Value, signature: &ValueType) -> bool {
    match signature {
        ValueType::Any => !value.is_null(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::Integer => value.as_i64().is_some(),
        ValueType::UnsignedInteger => value.as_u64().is_some(),
        ValueType::Double => value.as_f64().is_some(),
        ValueType::String => value.is_string(),
        ValueType::Nullable(null) => value.is_null() || validate_json_value(&value, null),
        ValueType::Union(union) => union.iter().any(|variant| validate_json_value(&value, variant)),
        ValueType::Tuple(tuple) => {
            let Some(arr) = value.as_array() else {
                return false;
            };

            arr.iter().zip_longest(tuple).all(|either| {
                let (left, right) = either.left_and_right();

                let Some(sig) = right else {
                    return false;
                };

                if left.is_none() {
                    return if let ValueType::Nullable(_) = sig {
                        true
                    } else {
                        false
                    }
                }

                validate_json_value(left.unwrap(), sig)
            })
        }
        ValueType::Array(ty) => {
            let Some(arr) = value.as_array() else {
                return false;
            };

            arr.iter().all(|i| validate_json_value(i, ty))
        }
        ValueType::Map(ty) => {
            let Some(obj) = value.as_object() else {
                return false;
            };

            obj.iter().all(|(_, v)| validate_json_value(v, ty))
        }
        ValueType::Object(entries) => {
            let Some(obj) = value.as_object() else {
                return false
            };

            for (key, sig) in entries {
                match sig {
                    ValueType::Nullable(null) => {
                        if let Some(val) = obj.get(key) {
                            if !validate_json_value(val, null) {
                                return false;
                            }
                        }
                    }
                    _ => {
                        if let Some(val) = obj.get(key) {
                            if !validate_json_value(val, sig) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            true
        }
    }
}