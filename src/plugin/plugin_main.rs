use compact_str::CompactString;
use mlua::{Lua, Result, StdLib};
use crate::plugin::action::PluginAction;
use std::thread;

fn spawn_lua_worker(script: String, action_tx: crossbeam::channel::Sender<PluginAction>) -> Result<()> {
    thread::spawn(move || {
        let lua = Lua::new();

        crate::plugin::policy::apply_restrictions(&lua, action_tx.clone(), &CompactString::from("")).expect("Something went wrong with applying resrtictions to plugins.");

        lua.load_std_libs(StdLib::ALL_SAFE).expect("Critical: Could not load Lua libs");

        let _ = crate::plugin::global_set::apply_globals(&lua, action_tx.clone());

        // Clean up, to free anything leftover from setting the security policy,
        // Such as the following scenario...
        // Say Table
        // `Table = { command = PointToSecurityRiskOne(), task = PointToSecurityRiskTwo() }`
        // Then we deem Table unsafe, and set it to Nil.
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

        loop {
            // Load and execute code
            // We compile it once, then run it in a loop
            let chunk = lua.load(&script);
            if let Err(e) = chunk.exec() {
                // Send the error back to your debug_logs!
                let _ = action_tx.send(PluginAction::DebugLog {
                    message:  e.to_string(),
                });
                // Avoid infinite high-speed error loops
                thread::sleep(std::time::Duration::from_secs(1));
            }

            thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    Ok(())
}

pub fn load_plugins(tx: crossbeam::channel::Sender<PluginAction>) -> Result<()> {
    // TODO: Make 3 worker threads, then make Lua give tasks
    let plugin = r#""#;

    let _ = spawn_lua_worker(plugin.to_string(), tx);

    Ok(())
}
