use mlua::prelude::*;

#[mlua::lua_module]
fn test_lua_module(lua: &Lua) -> LuaResult<LuaTable> {
    let global = lua.globals();

    global.set("Init", lua.create_function(|_, _: ()| {
        println!("native init function wooo!");

        Ok(())
    })?)?;

    // not actually exporting anything of value, just there to modify global table
    Ok(lua.create_table()?)
}