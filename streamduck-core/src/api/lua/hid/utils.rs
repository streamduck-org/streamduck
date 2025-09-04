use itertools::Itertools;
use mlua::{AnyUserData, FromLua, Lua, LuaSerdeExt, MetaMethod, Table, UserData, UserDataMethods, Value, Variadic};
use std::ops::{Deref, DerefMut};

pub struct LuaByteArray {
    pub data: Vec<u8>
}

impl FromIterator<u8> for LuaByteArray {
    fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
        Self {
            data: Vec::from_iter(iter)
        }
    }
}

impl From<LuaByteArray> for Vec<u8> {
    fn from(value: LuaByteArray) -> Self {
        value.data
    }
}

impl From<Vec<u8>> for LuaByteArray {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value
        }
    }
}

impl Deref for LuaByteArray {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for LuaByteArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl FromLua for LuaByteArray {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(userdata) => Ok(userdata.take::<LuaByteArray>()?),
            Value::Table(_) => Ok(lua.from_value::<Vec<u8>>(value)?.into()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaByteArray".to_string(),
                message: None,
            }),
        }
    }
}

impl UserData for LuaByteArray {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("padLeft", |_lua, (mut this, amount): (LuaByteArray, usize)| {
            let mut padding = vec![0u8; amount];
            padding.append(&mut *this);

            Ok(LuaByteArray::from(padding))
        });

        methods.add_function("padRight", |_lua, (mut this, amount): (LuaByteArray, usize)| {
            this.extend((0..amount).into_iter().map(|_| 0));

            Ok(LuaByteArray::from(this))
        });

        methods.add_function("append", |_lua, (mut this, rhs): (LuaByteArray, LuaByteArray)| {
            this.extend(rhs.data.into_iter());

            Ok(this)
        });

        methods.add_function("appendBuf", |_lua, (mut this, bytes): (LuaByteArray, Variadic<u8>)| {
            this.extend(bytes.into_iter());

            Ok(this)
        });

        methods.add_function("repeated", |_lua, (this, amount): (LuaByteArray, usize)| {
            let size = this.len();
            Ok(this.data.into_iter().cycle().take(size * amount).collect::<LuaByteArray>())
        });

        methods.add_function("chunks", |_lua, (this, chunk_size): (LuaByteArray, usize)| {
            Ok(this.data.into_iter()
                .chunks(chunk_size)
                .into_iter()
                .map(|c| c.into_iter().collect())
                .collect::<Vec<LuaByteArray>>())
        });

        methods.add_function("toTable", |_lua, this: LuaByteArray| {
            Ok(this.data)
        });

        methods.add_meta_method(MetaMethod::Index, |_lua, this, index: usize| {
            Ok(this.data.get(index - 1).copied())
        });

        methods.add_meta_method_mut(MetaMethod::NewIndex, |_lua, this, (index, value): (usize, u8)| {
            let index = index - 1;

            if index < this.len() {
                this[index] = value;
            } else {
                this.resize(index + 1, 0);
                this[index] = value;
            }

            Ok(())
        });

        methods.add_meta_method(MetaMethod::Len, |_lua, this, _: ()| {
            Ok(this.len())
        });

        methods.add_meta_function(MetaMethod::IPairs, |lua, this: AnyUserData| {
            let iter_func = lua.create_function(|_lua, (this, index): (AnyUserData, usize)| {
                let this = this.borrow::<LuaByteArray>()?;

                let next = index + 1;

                if next > this.len() {
                    Ok((None, None))
                } else {
                    Ok((Some(next), Some(this[index])))
                }
            })?;

            Ok((iter_func, this, 0))
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, this: AnyUserData| {
            let next_func = lua.create_function(|_lua, (this, index): (AnyUserData, Option<usize>)| {
                let this = this.borrow::<LuaByteArray>()?;

                match index {
                    Some(index) => {
                        let next = index + 1;

                        if next > this.len() {
                            Ok((None, None))
                        } else {
                            Ok((Some(next), Some(this[index])))
                        }
                    },
                    None => {
                        if !this.is_empty() {
                            Ok((Some(1), Some(this[0])))
                        } else {
                            Ok((None, None))
                        }
                    }
                }
            })?;

            Ok((next_func, this, Value::Nil))
        })
    }
}

pub fn buf_util(lua: &Lua) -> Result<Table, mlua::Error> {
    let buf = lua.create_table()?;

    buf.set("__call", lua.create_function(|_lua, (_, values): (Table, Variadic<u8>)| {
        Ok(values.into_iter().collect::<LuaByteArray>())
    })?)?;

    buf.set("convert", lua.create_function(move |_lua, values: Vec<u8>| {
        Ok(values.into_iter().collect::<LuaByteArray>())
    })?)?;

    buf.set_metatable(Some(buf.clone()))?;
    Ok(buf)
}

pub fn byte_util(lua: &Lua) -> Result<Table, mlua::Error> {
    let byte = lua.create_table()?;

    byte.set("i8", lua.create_function(|_, byte: u8| {
        Ok(byte as i8)
    })?)?;

    byte.set("u16", lua.create_function(|_, (b1, b2): (u8, u8)| {
        Ok(u16::from_le_bytes([b1, b2]))
    })?)?;

    byte.set("u16be", lua.create_function(|_, (b1, b2): (u8, u8)| {
        Ok(u16::from_be_bytes([b1, b2]))
    })?)?;

    byte.set("i16", lua.create_function(|_, (b1, b2): (u8, u8)| {
        Ok(i16::from_le_bytes([b1, b2]))
    })?)?;

    byte.set("i16be", lua.create_function(|_, (b1, b2): (u8, u8)| {
        Ok(i16::from_be_bytes([b1, b2]))
    })?)?;

    byte.set("u32", lua.create_function(|_, (b1, b2, b3, b4): (u8, u8, u8, u8)| {
        Ok(u32::from_le_bytes([b1, b2, b3, b4]))
    })?)?;

    byte.set("u32be", lua.create_function(|_, (b1, b2, b3, b4): (u8, u8, u8, u8)| {
        Ok(u32::from_be_bytes([b1, b2, b3, b4]))
    })?)?;

    byte.set("i32", lua.create_function(|_, (b1, b2, b3, b4): (u8, u8, u8, u8)| {
        Ok(i32::from_le_bytes([b1, b2, b3, b4]))
    })?)?;

    byte.set("i32be", lua.create_function(|_, (b1, b2, b3, b4): (u8, u8, u8, u8)| {
        Ok(i32::from_be_bytes([b1, b2, b3, b4]))
    })?)?;

    byte.set("u64", lua.create_function(
        |_, (b1, b2, b3, b4, b5, b6, b7, b8): (u8, u8, u8, u8, u8, u8, u8, u8)| {
            Ok(u64::from_le_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
        }
    )?)?;

    byte.set("u64be", lua.create_function(
        |_, (b1, b2, b3, b4, b5, b6, b7, b8): (u8, u8, u8, u8, u8, u8, u8, u8)| {
            Ok(u64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
        }
    )?)?;

    byte.set("i64", lua.create_function(
        |_, (b1, b2, b3, b4, b5, b6, b7, b8): (u8, u8, u8, u8, u8, u8, u8, u8)| {
            Ok(i64::from_le_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
        }
    )?)?;

    byte.set("i64be", lua.create_function(
        |_, (b1, b2, b3, b4, b5, b6, b7, b8): (u8, u8, u8, u8, u8, u8, u8, u8)| {
            Ok(i64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
        }
    )?)?;

    byte.set("as2Bytes", lua.create_function(|_, val: u16| {
        let bytes = val.to_le_bytes();
        Ok((bytes[0], bytes[1]))
    })?)?;

    byte.set("as4Bytes", lua.create_function(|_, val: u32| {
        let bytes = val.to_le_bytes();
        Ok((bytes[0], bytes[1], bytes[2], bytes[3]))
    })?)?;

    byte.set("as8Bytes", lua.create_function(|_, val: u64| {
        let bytes = val.to_le_bytes();
        Ok((bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]))
    })?)?;

    byte.set("as2BEBytes", lua.create_function(|_, val: u16| {
        let bytes = val.to_be_bytes();
        Ok((bytes[0], bytes[1]))
    })?)?;

    byte.set("as4BEBytes", lua.create_function(|_, val: u32| {
        let bytes = val.to_be_bytes();
        Ok((bytes[0], bytes[1], bytes[2], bytes[3]))
    })?)?;

    byte.set("as8BEBytes", lua.create_function(|_, val: u64| {
        let bytes = val.to_be_bytes();
        Ok((bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]))
    })?)?;

    Ok(byte)
}