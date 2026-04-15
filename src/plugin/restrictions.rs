use mlua::{Lua, Result, Value::Nil, Value, Table, Error};

pub fn apply_restrictions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Included in ALL_SAFE; This is something unsafe for us
    globals.set("package", Nil)?;

    globals.set("debug", Nil)?;

    // Printing shifts up the screen, which we *DON'T* want
    globals.set("print", Nil)?;

    // Help to prevent version specific exploits
    globals.set("_VERSION", Nil)?;

    // Fail on value not existing in table / global variables
    let strict_index = lua.create_function(|_, (_table, key): (Value, String)| {
        Err::<Value, _>(Error::RuntimeError(format!(
            "RuntimeError: Variable or Field '{}' does not exist.",
            key
        )))
    })?;

    // Apply global protection
    let global_mt = lua.create_table()?;
    global_mt.set("__index", strict_index.clone())?;
    let _ = globals.set_metatable(Some(global_mt));

    // Users use this to make their own tables error on missing data
    let create_strict_table = lua.create_function(move |lua, initial_data: Table| {
        let mt = lua.create_table()?;
        mt.set("__index", strict_index.clone())?;
        let _ = initial_data.set_metatable(Some(mt));
        Ok(initial_data)
    })?;
    globals.set("StrictTable", create_strict_table)?;

    Ok(())
}
