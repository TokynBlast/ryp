use mlua::{Lua, Result, StdLib};
use crate::plugin::lua_io;
use mlua::Value::Nil;

pub fn load_plugins() -> Result<()> {
    let lua = Lua::new();

    lua.load_std_libs(StdLib::ALL_SAFE)?;

    let globals = lua.globals();

    // Printing shifts up the screen, which we *DON'T* want
    globals.set("print", Nil)?;

    // Help to prevent version specific exploits
    globals.set("_VERSION", "")?;

    // Opens a file, for functions to perform on
    let open_fn = lua.create_function(|_, path: String| {
        lua_io::open_file(path)
    })?;
    globals.set("open", open_fn)?;

    // TODO: Make 3 worker threads, then make Lua give tasks
    lua.load(r#"
        print("This still needs to be done")
    "#).exec()?;
    Ok(())
}
