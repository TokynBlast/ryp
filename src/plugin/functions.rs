use mlua::{Lua, Result, StdLib};
use crate::plugin::lua_io;

pub fn load_plugins() -> Result<()> {
    let lua = Lua::new();

    crate::plugin::restrictions::apply_restrictions(&lua).expect("Something went wrong with applying resrtictions to plugin Lua.");

    lua.load_std_libs(StdLib::ALL_SAFE)?;

    let globals = lua.globals();

    // Opens a file, for functions to perform on
    let open_fn = lua.create_function(|_, path: String| {
        lua_io::open_file(path)
    })?;
    globals.set("open", open_fn)?;

    // TODO: Hook this up to real cursor
    globals.set("cursor_x", 0)?;
    globals.set("cursor_y", 0)?;

    // TODO: Make 3 worker threads, then make Lua give tasks
    lua.load(r#"
        print("This still needs to be done")
    "#).exec()?;
    Ok(())
}
