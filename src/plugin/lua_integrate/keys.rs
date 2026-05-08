use compact_str::CompactString;
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, StrResponder};

pub fn integrate_keys(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let tx_clone = tx.clone();

    let responder = Arc::new(StrResponder {
        string: Mutex::new(CompactString::default()),
        signal: Condvar::new(),
    });

    let key_fn = lua.create_function(move |lua, ()| {
        let responder_clone = responder.clone();
        let _ = tx_clone.send(PluginAction::GetKeyPress {
            responder: responder_clone.clone()
        });

        let mut lock = responder_clone.string.lock();
        responder_clone.signal.wait(&mut lock);

        let info = lock.clone();
        lua.to_value(&info.to_string())
    })?;

    lua.globals().set("key", key_fn)?;

    Ok(())
}
