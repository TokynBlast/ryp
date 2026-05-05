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
    cursor::integrate_cursor_pos(lua, &tx)?;

    Ok(())
}
