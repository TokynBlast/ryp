use compact_str::CompactString;
use mlua::{Lua, Result, StdLib};
use crate::plugin::action::PluginAction;
use std::{fs, path::PathBuf, thread};
use rayon;

fn spawn_lua_worker(plugin_path: PathBuf, action_tx: crossbeam_channel::Sender<PluginAction>) -> Result<()> {
    let script = fs::read_to_string(PathBuf::from(&plugin_path).join("src").join("main.lua"))?;
    let plugin_policy = serde_json::Value::from(fs::read_to_string(PathBuf::from(&plugin_path).join("config"))?);
    if !script.is_empty() {
        rayon::spawn(move || {
            // Since an empty script is wasted work, we just do nothing with it
            let lua = Lua::new();

       crate::plugin::policy::apply_restrictions(&lua, action_tx.clone(), &plugin_policy).expect("Something went wrong with applying resrtictions to plugins.");

            lua.load_std_libs(StdLib::ALL_SAFE).expect("Critical: Could not load Lua libs");

            let _ = crate::plugin::global_set::apply_globals(&lua, action_tx.clone());

            // Clean up, to free anything leftover from setting the security policy,
            // Such as the following scenario...
            // `Table` is a pre-defined global that was included. Then we set it to Nil.
            // The contents of Table still exist, taking up memory.

            // Called twice as suggested by documentation
            let _ = lua.gc_collect();
            let _ = lua.gc_collect();

            // Set a 10MB limit
            // It's here, so that during startup, we don't somehow hit it, triggering an unnecesarry collection
            // Especially in the future
            if let Err(e) = lua.set_memory_limit(10 * 1024 * 1024) {
                let _ = action_tx.send(PluginAction::DebugLog {
                    message: format!("Memory limit error: {}", e)
                });
            }

            // Compile the script
            if let Err(e) = lua.load(&script).exec() {
                let _ = action_tx.send(PluginAction::DebugLog { message: format!("Script Load Error: {}", e) });
                return; // Can't continue if the script is broken
            }

            // After compiling the script, it is useless. clearing it frees a variable amount of memory
            drop(script);
            drop(plugin_policy);

            // Run init (if it exists)
            let plugin_init: Option<mlua::Function> = lua.globals().get("init").unwrap();
            if let Some(f) = plugin_init {
                let _ = f.call::<()>(());
            }

            let run_fn: Option<mlua::Function> = lua.globals().get("run").ok();
            loop {
                if let Some(ref f) = run_fn {
                    if let Err(e) = f.call::<()>(()) {
                        let _ = action_tx.send(PluginAction::DebugLog { message: e.to_string() });
                    }
                }
            }
        });
    }
    Ok(())
}

pub fn load_plugins(plugin_path: CompactString, tx: crossbeam_channel::Sender<PluginAction>) -> Result<()> {
    // We don't have to worry about it not existing, as we create it if it doesn't, and don't run this if it doesn't!
    let entries = std::fs::read_dir(&plugin_path).unwrap();

    for entry in entries.flatten() {
        let path = entry.path();

        // 2. We only care about directories (each directory is a plugin)
        if path.is_dir() {
            let _ = spawn_lua_worker(path, tx.clone());
        }
    }
    Ok(())
}
