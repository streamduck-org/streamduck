use mlua::Table;

pub trait LuaTablesUtils {
    fn contains_keys(&self, keys: &[&str]) -> Result<bool, mlua::Error>;
}

impl LuaTablesUtils for Table {
    fn contains_keys(&self, keys: &[&str]) -> Result<bool, mlua::Error> {
        for key in keys {
            if !self.contains_key(*key)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}