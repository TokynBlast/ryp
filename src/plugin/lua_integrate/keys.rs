use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, CharResponder};

pub fn integrate_keys(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let tx_clone = tx.clone();

    let responder = Arc::new(CharResponder {
        c: Mutex::new(None),
        signal: Condvar::new(),
    });

    let key_fn = lua.create_function(move |lua, ()| {
        let responder_clone = responder.clone();
        let _ = tx_clone.send(PluginAction::GetKeyPress {
            responder: responder_clone.clone()
        });

        let mut lock = responder_clone.c.lock();
        if lock.is_none() {
            responder_clone.signal.wait(&mut lock);
        }

        let info = lock.take();
        lua.to_value(&info)
    })?;

    lua.globals().set("key", key_fn)?;

    Ok(())
}
