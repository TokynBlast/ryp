use compact_str::CompactString;
use mlua::{Lua, Result, Value::{self, Nil}};
use crate::plugin::action::PluginAction;
use serde_json::{self, json};
use std::path::PathBuf;
use std::fs;

#[inline]
pub fn apply_restrictions(lua: &Lua, tx: crossbeam::channel::Sender<crate::plugin::action::PluginAction>, plugin_name: &CompactString) -> Result<()> {
    let mut defaults = json!({
        "networking": false,
        "read stdout": false,
        "io operations": true,
        "limited io operations": true,
        "unlimited io operations": false,
        "stdin access": false,
        "stdout access": false,
        "limited io locations": [],
        "make threads": false,
    });

    // If this fails, we can't allow IO, or we should crash :)
    defaults["limited io locations"] = if cfg!(windows) {
        let base = std::env::var("APPDATA");
        let path = PathBuf::from(base.unwrap()).join(format!("ryp\\plugins\\{}", plugin_name));
        json!([path])
    } else {
        let base = std::env::var("HOME");
        let path = PathBuf::from(base.unwrap()).join(format!(".ryp/plugins/{}", plugin_name));
        json!([path])
    };

    // TODO: Make it so that it uses either what was in the file, or the default, and if anything changed, write to the config
    let plugin_config = PathBuf::from(defaults["limited io loactions"][0].to_string());
    let _ = plugin_config.join(PathBuf::from("config"));

    // This is globbed together, to avoid naming confusion...
    //   There aren't many names we have available, before it becomes redundant and hard to maintain
    let configuration =
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(plugin_config)?
        ).unwrap();

    let globals = lua.globals();

    // Included in ALL_SAFE; This is something unsafe for us
    globals.set("package", Nil)?;
    globals.set("debug", Nil)?;
    globals.set("loadfile", Nil)?;
    globals.set("collectgarbage", Nil)?;
    globals.set("_VERSION", Nil)?;
    globals.set("table", Nil)?;
    globals.set("require", Nil)?; // TODO: Make this so we can only load some lua files, excluding luac files
    globals.set("warn", Nil)?;

    // There are a couple from IO that we keep...
    // However, we drop nearly every single one
    if serde_json::Value::as_bool(&configuration["unlimited io operations"])
        .expect("Could not determine whether or not to allow IO operations") {
        globals.set("io.close", Nil)?;
        globals.set("io.tmpfile", Nil)?;
        globals.set("io.stderr", Nil)?;
        globals.set("io.flush", Nil)?;
        if serde_json::Value::as_bool(&configuration["stdout access"])
            .expect("Could not determine whether or not to allow access to stdout") {
            globals.set("io.stdout", Nil)?;
        }
        if serde_json::Value::as_bool(&configuration["stdin access"])
            .expect("Could not determine whether or not to allow access to stdin") {
            globals.set("io.stdin", Nil)?;
        }
        globals.set("io.output", Nil)?;
        globals.set("io.read", Nil)?;
        globals.set("io.write", Nil)?;
        globals.set("io.open", Nil)?;
        globals.set("io.type", Nil)?;
        globals.set("io.popen", Nil)?;
        globals.set("io.lines", Nil)?;
    }
    globals.set("assert", Nil)?;
    globals.set("rawset", Nil)?;
    globals.set("getmetatable", Nil)?;
    globals.set("setmetatable", Nil)?;
    globals.set("arg", Nil)?;

    if serde_json::Value::as_bool(&configuration["make threads"])
        .expect("Could not determine whether or not to allow making threads") {
        globals.set("coroutine", Nil)?;
    }


    // Consider: rawlen, rawequals
    // arg     table
    // xpcall  function
    // pcall   function
    // error   function
    // select  function
    // ipairs  function


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

    Ok(())
}
