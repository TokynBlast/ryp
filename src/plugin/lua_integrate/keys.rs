use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, CharResponder};

fn get_key_press(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, get_table: &mlua::Table) -> Result<(), mlua::Error> {
    let responder = Arc::new(CharResponder {
        c: Mutex::new(None),
        signal: Condvar::new(),
    });
    let responder_clone = responder.clone();
    let tx_clone = tx.clone();
    get_table.set("key",
        lua.create_function(move |lua, ()| {
              let _ = tx_clone.send(PluginAction::GetKeyPress { responder: responder_clone.clone() });

              let mut lock = responder_clone.c.lock();
              if lock.is_none() {
                  responder_clone.signal.wait(&mut lock);
              }

              let info = lock.take();
              lua.to_value(&info)
        })?
    )?;
    Ok(())
}

// wiring to expose register_shortcut to Lua
pub fn integrate_keys(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let key_table = lua.create_table()?;
    get_key_press(lua, tx, &key_table)?;
    let proxy = lua.create_table()?;
    let metatable = lua.create_table()?;

    let internal_editor = key_table.clone();
    metatable.set("__index", lua.create_function(move |_, (_, key): (mlua::Value, String)| {
        internal_editor.get::<mlua::Value>(key)
    })?)?;

    proxy.set_metatable(Some(metatable))?;

    lua.globals().set("editor", proxy)?;
    Ok(())
}
