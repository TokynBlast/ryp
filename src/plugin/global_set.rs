use mlua;
use crate::plugin::action::PluginAction;

pub fn apply_globals(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let globals = lua.globals();
    // Opens a file, for functions to perform on
    globals.set("open",
        lua.create_function(|_, path: String| {
            lua_io::open_file(path)
        })?
    )?;

    /*globals.set("settings.new",
        lua.create_function(move|_, (name, default): (mlua::String, mlua::Value)| {
            crate::plugin::lua_integrate::settings::add_setting(name, default, app.clone())
        })?
    )?;*/

    // TODO: Hook this up to real cursor
    globals.set("cursor_x", 0)?;
    globals.set("cursor_y", 0)?;

    Ok(())
}
