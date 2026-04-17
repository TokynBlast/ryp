use mlua::{Error, Lua, Result, Table, Value::{self, Nil}};
use crate::plugin::action::PluginAction;

pub fn apply_restrictions(lua: &Lua, tx: &crossbeam::channel::Sender<crate::plugin::action::PluginAction>) -> Result<()> {
    let globals = lua.globals();

    // Included in ALL_SAFE; This is something unsafe for us
    globals.set("package", Nil)?;

    globals.set("debug", Nil)?;
    globals.set("load", Nil)?;
    globals.set("dofile", Nil)?;
    globals.set("collectgarbage", Nil)?;

    // Printing shifts up the screen, which we *DON'T* want
    // Instead, we offer printing, but contained :)
    let tx_print = tx.clone();

    // Redirects Lua `print()` function to a debug console
    // This is apart of the policy, since print must go to the debug console, and going elsewhere is not accepted.
    let debug_print_fn = lua.create_function(move |_, args: mlua::Variadic<mlua::Value>| {
        let msg = args
            .iter()
            .map(|v| match v {
                // .ok() converts Result to Option, allowing unwrap_or to take a &str
                Value::String(s) => s.to_str().ok().as_deref().unwrap_or("").to_string(),
                Value::Nil => "nil".to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Number(n) => n.to_string(),
                any => format!("{:?}", any),
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Match your existing variant: DebugLog { message: String }
        let _ = tx_print.send(PluginAction::DebugLog { message: msg });

        Ok(())
    })?;

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
