use mlua::{Lua, Result, Table};
use std::path;
use std::io::{self, Read};
use std::fs;
use std::io;
use std::path::PathBuf;
use lua_io;

#[inline(always)]


fn query_installed() {
    #[cfg(windows)]
    plugin_dir = "C:/.ryp_plugins";
    #[cfg(not(windows))]
    plugin_dir = "./.ryp_plugins";

    // Even though because this function is being called, we assume
    // the folder doesn't exist... This is for safety, in the case it doesn't-
    match fs::metadata(plugin_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "plugin path exists but is not a directory",
                ));
            }
            // Directory exists: count entries and return that count
            let count = fs::read_dir(plugin_dir)?.count();
            Ok(count)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            // Doesn't exist: create directory and return 0, since we just made it
            fs::create_dir_all(plugin_dir)?;
            Ok(0)
        }
        Err(err) => Err(err),
    }
}

fn load_plugins() -> Result<()> {
    let lua = Lua::new();

    // opens a file
    let open_file = lua.create_function(|_, (directory, ): (PathBuf, String)| Ok(query_installed))?;
    let open_fn = lua.create_function(|_, path: String| {
        open_file(path).map_err(Error::external)
    })?;

    lua.globals()
        .set("InstallQuery", query_installed)
        .set("open", lua_io::open_fn)?;

    // TODO: Load each into their own thread
    //       Or, we pool them all into 3 threads
    lua.load(r#"
        print("This still needs to be done")
    "#).exec()?;

    println!("final counter = {}", *counter.lock().unwrap());
    Ok(())
}
