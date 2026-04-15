use mlua::{Lua, Result};
use std::io::{self};
use std::fs;
use crate::plugin::lua_io;

#[inline(always)]
fn query_installed() -> io::Result<usize> {
    let plugin_dir = if cfg!(windows) {
        "C:/.ryp_plugins"
    } else {
        "~/.ryp_plugins"
    };

    match fs::metadata(plugin_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, "plugin path exists but is not a directory"));
            }
            let count = fs::read_dir(plugin_dir)?.count();
            Ok(count)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(plugin_dir)?;
            Ok(0)
        }
        Err(e) => Err(e),
    }
}

pub fn load_plugins() -> Result<()> {
    let lua = Lua::new();

    // Safety; Printing shifts up the screen, which we *DON'T* want
    globals.set("print", mlua::Value::Nil)?;

    // wrap query_installed as a lua function — no args, returns count
    let query_installed_fn = lua.create_function(|_, ()| {
        query_installed().map_err(mlua::Error::external)
    })?;

    // wrap open_file as a lua function — takes a path string
    let open_fn = lua.create_function(|_, path: String| {
        lua_io::open_file(path)
    })?;

    let globals = lua.globals();
    globals.set("InstallQuery", query_installed_fn)?;
    globals.set("open", open_fn)?;

    // TODO: Load each into their own thread
    //       Or, we pool them all into 3 threads
    lua.load(r#"
        print("This still needs to be done")
    "#).exec()?;
    Ok(())
}
