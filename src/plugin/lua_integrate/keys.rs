use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, StrResponder};

pub fn integrate_keys(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let tx_clone = tx.clone();

    let responder = Arc::new(StrResponder {
        string: Mutex::new(None),
        signal: Condvar::new(),
    });

    let key_fn = lua.create_function(move |lua, ()| {
        let responder_clone = responder.clone();
        let _ = tx_clone.send(PluginAction::GetKeyPress {
            responder: responder_clone.clone()
        });

        let mut lock = responder_clone.string.lock();
        if lock.is_none() {
            responder_clone.signal.wait(&mut lock);
        }

        let info = lock.clone();
        if info.is_some() {
            lua.to_value(&info.unwrap().to_string())
        } else {
            lua.to_value(&mlua::Nil)
        }
    })?;

    lua.globals().set("key", key_fn)?;

    Ok(())
}
