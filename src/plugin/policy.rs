use mlua::{Lua, Result, Value::{self, Nil}};
use crate::plugin::action::PluginAction;

#[inline]
pub fn apply_restrictions(lua: &Lua, tx: crossbeam_channel::Sender<crate::plugin::action::PluginAction>, policy: &serde_json::Value) -> Result<()> {
    // "networking": false,
    // "read stdout": false,
    // "io operations": true,
    // "limited io operations": true,
    // "unlimited io operations": false,
    // "stdin access": false,
    // "stdout access": false,
    // "limited io locations": [],
    // "make threads": false,
    // "forced garbage collection": false,

    let globals = lua.globals();

    let not_allowed = |key: &str| -> bool {
        policy.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false) // If it's missing, we assume it's not allowed, and that the plugin is from earlier versions
    };

    // Included in ALL_SAFE; This is something unsafe for us
    globals.set("package", Nil)?;
    globals.set("debug", Nil)?;
    globals.set("loadfile", Nil)?;
    if not_allowed("forced garbage collection") {
        globals.set("collectgarbage", Nil)?;
    }
    globals.set("_VERSION", Nil)?;
    globals.set("require", Nil)?; // TODO: Make this so we can only load some lua files, excluding luac files
    globals.set("warn", Nil)?;

    // There are a couple from IO that we keep...
    // However, we drop nearly every single one
    if not_allowed("unlimited io operations") {
        globals.set("io.tmpfile", Nil)?;
        globals.set("io.stderr", Nil)?;
        globals.set("io.flush", Nil)?;
        if not_allowed("stdout access") {
            globals.set("io.stdout", Nil)?;
        }
        if not_allowed("stdin access") {
            globals.set("io.stdin", Nil)?;
        }
        globals.set("io.output", Nil)?;

        // These are replaced by our own
        globals.set("io.read", Nil)?;
        globals.set("io.write", Nil)?;
        globals.set("io.open", Nil)?;
        globals.set("io.lines", Nil)?;
        globals.set("io.close", Nil)?;

        globals.set("io.type", Nil)?;
        globals.set("io.popen", Nil)?;
    }
    globals.set("assert", Nil)?;
    globals.set("rawset", Nil)?;
    globals.set("getmetatable", Nil)?;
    globals.set("setmetatable", Nil)?;
    globals.set("arg", Nil)?;

    if not_allowed("make threads") {
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

        let _ = tx.send(PluginAction::DebugLog { message: msg });

        Ok(())
    })?;

    globals.set("print", debug_print_fn)?;

    Ok(())
}
