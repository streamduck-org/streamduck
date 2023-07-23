use std::error::Error;
use rmpv::Value;
use serde::Serialize;
use sha2::Sha256;
use sha2::Digest;
use tokio::task::block_in_place;

/// Takes in anything that could be turned into Value using Into trait, and returns a Value
#[macro_export]
macro_rules! msgpack {
    ($e:expr) => {
        rmpv::Value::from($e)
    };
}

/// Takes in anything that could be turned into Value using Into trait, and returns a vector with Values
#[macro_export]
macro_rules! msgvec {
    ($( $e:expr ),*) => {
        vec![$(rmpv::Value::from($e)),*]
    };
}

/// Takes in anything that could be turned into Value using Into trait, and returns a slice with Values
#[macro_export]
macro_rules! msgslice {
    ($( $e:expr ),*) => {
        &[$(rmpv::Value::from($e)),*]
    };
}

/// Traverses MSGPack value and returns reference to the target
pub fn traverse_msgpack<'a>(value: &'a Value, path: &[Value]) -> Option<&'a Value> {
    let mut target = value;

    for crumb in path {
        match target {
            Value::Array(arr) => {
                let Some(index) = crumb.as_i64() else {
                    return None;
                };

                target = arr.get(index as usize)?;
            }

            Value::Map(map) => {
                target = &map.iter().find(|(key, _)| key == crumb)?.1;
            }

            _ => return None,
        }
    }

    Some(target)
}

/// Traverses MSGPack value with mutable references and returns reference to the target
pub fn traverse_msgpack_mut<'a>(value: &'a mut Value, path: &[Value]) -> Option<&'a mut Value> {
    let mut target = value;

    for crumb in path {
        match target {
            Value::Array(arr) => {
                let Some(index) = crumb.as_i64() else {
                    return None;
                };

                target = arr.get_mut(index as usize)?;
            }

            Value::Map(map) => {
                target = &mut map.iter_mut().find(|(key, _)| key == crumb)?.1;
            }

            _ => return None,
        }
    }

    Some(target)
}

/// Hashes anything that can be serialized into SHA-256
pub fn sha256_digest(serializable: impl Serialize) -> Result<String, rmp_serde::encode::Error> {
    let msg = block_in_place(move || { rmp_serde::to_vec(&serializable) })?;

    let mut hasher = Sha256::new();
    hasher.update(msg.as_slice());
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

/// Attempts to serialize input into a value
pub fn serialize_into_value(serializable: impl Serialize) -> Result<Value, Box<dyn Error>> {
    let data = rmp_serde::to_vec_named(&serializable)?;
    Ok(rmp_serde::from_slice(data.as_slice())?)
}