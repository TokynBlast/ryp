use mlua;
use crate::plugin::action::PluginAction;

pub fn apply_globals(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let globals = lua.globals();

    // Opens a file, for functions to perform on
    globals.set("open",
        lua.create_function(|_, path: String| {
            crate::plugin::lua_io::open_file(path)
        })?
    )?;

    let tx_insert = tx.clone();
    globals.set("insert_text",
        lua.create_function(move |_, text: char| {
            let _ = tx_insert.send(PluginAction::InsertText { text });
            Ok(())
        })?
    )?;

    crate::plugin::lua_integrate::settings::integrate_settings(lua, tx)?;

    // TODO: Hook this up to real cursor
    globals.set("cursor_x", 0)?;
    globals.set("cursor_y", 0)?;

    Ok(())
}
