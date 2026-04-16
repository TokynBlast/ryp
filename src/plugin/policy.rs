use mlua::{Lua, Result, Value::Nil, Value, Table, Error, Variadic};

/// Redirects Lua `print()` function to
fn debug_print(_lua: &Lua, args: Variadic<Value>) -> Result<()> {
    // Convert all arguments to strings and join them with tabs (standard print behavior)
    let output = args
        .iter()
        .map(|v| match v {
            Value::String(s) => s.to_string_lossy(),
            Value::Nil => "nil".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Number(n) => n.to_string(),
            _ => format!("{:?}", v), // Fallback for tables, functions, etc.
        })
        .collect::<Vec<_>>()
        .join("\t");

    todo!("Add the debug console");

    Ok(())
}

pub fn apply_restrictions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Included in ALL_SAFE; This is something unsafe for us
    globals.set("package", Nil)?;

    globals.set("debug", Nil)?;

    // Printing shifts up the screen, which we *DON'T* want
    // Instead, we offer printing, but contained :)
    let debug_print_fn = lua.create_function(debug_print)?;
    globals.set("print", debug_print_fn)?;

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
