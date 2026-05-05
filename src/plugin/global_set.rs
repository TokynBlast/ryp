use mlua;
use crate::plugin::{action::PluginAction, lua_integrate::*};

pub fn apply_globals(lua: &mlua::Lua, tx: crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let globals = lua.globals();

    // Opens a file, for functions to perform on
    globals.set("open",
        lua.create_function(|_, path: String| {
            crate::plugin::lua_io::open_file(path)
        })?
    )?;

    settings::integrate_settings(lua, &tx)?;
    editor::integrate_editor(lua, &tx)?;
    keys::integrate_keys(lua, &tx)?;

    // TODO: Hook this up to real cursor
    globals.set("cursor_x", 0)?;
    globals.set("cursor_y", 0)?;

    Ok(())
}
