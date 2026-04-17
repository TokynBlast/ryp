use mlua;
use crate::plugin::action::PluginAction;

pub fn apply_globals(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let globals = lua.globals();
    let settings_table = lua.create_table()?;

    // Opens a file, for functions to perform on
    globals.set("open",
        lua.create_function(|_, path: String| {
            crate::plugin::lua_io::open_file(path)
        })?
    )?;

    // We clone the sender for each function
    let tx_add:
        crossbeam_channel::Sender<PluginAction> = tx.clone();

    settings_table.set("add",
        lua.create_function(move |_, (name, value): (String, mlua::Value)| {
            let _ = tx_add.send(PluginAction::SetSetting { name, value });
            Ok(())
        })?
    )?;

    let tx_get = tx.clone();
    settings_table.set("get",
        lua.create_function(move |_, name: String| {
            let name_on_error: String = name.clone();

            let (tx_respond, rx_respond) = oneshot::channel::<String>();
            // Send request for value
            let _ = tx_get.send(PluginAction::GetSettingValue { name, tx_respond });
            // Wait for value
            let info = rx_respond.try_recv().map_err(|_| mlua::Error::RuntimeError(format!("Fatal error: could not get value {}", name_on_error).into()))?;

            Ok(info)
        })?
    )?;

    let tx_insert = tx.clone();
    globals.set("insert_text",
        lua.create_function(move |_, (text, x, y): (String, usize, usize)| {
            let _ = tx_insert.send(PluginAction::InsertText { text, x, y });
            Ok(())
        })?
    )?;

    // TODO: This should get the config :)
    globals.set("get_info",
        lua.create_function(move |_, ()| {
            let (tx_respond, rx_respond) = oneshot::channel::<String>();

            // Send the request with the "return address"
            tx.send(PluginAction::GetAppInfo { respond_to: tx_respond }).ok();

            // Wait for the response (blocking the Lua script briefly)
            let info = rx_respond.try_recv().map_err(|_| mlua::Error::RuntimeError("Fatal error in Rust!".into()))?;

            Ok(info)
        })?
    )?;

    // TODO: Hook this up to real cursor
    globals.set("cursor_x", 0)?;
    globals.set("cursor_y", 0)?;
    globals.set("settings", settings_table)?;

    Ok(())
}
