use crate::plugin::action::{PluginAction, UsizeResponder};
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};

fn get_cursor_x(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_get: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table_get.set("x",
        lua.create_function(move |lua, ()| {
            let _ = tx.send(PluginAction::GetCursorX { responder: responder_clone.clone() });
            let mut lock = responder_clone.number.lock();
            responder_clone.signal.wait(&mut lock);

            let info = *lock;
            lua.to_value(&info)
        })?
    )?;
    Ok(())
}

fn get_cursor_y(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_get: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table_get.set("y",
        lua.create_function(move |lua, ()| {
            let _ = tx.send(PluginAction::GetCursorY { responder: responder_clone.clone() });
            let mut lock = responder_clone.number.lock();
            responder_clone.signal.wait(&mut lock);

            let info = *lock;
            lua.to_value(&info)
        })?
    )?;
    Ok(())
}

pub fn integrate_cursor_pos(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let cursor_table = lua.create_table()?;
    let cursor_table_set = lua.create_table()?;
    let cursor_table_get = lua.create_table()?;

    let responder = Arc::new(UsizeResponder {
        number: Mutex::new(0),
        signal: Condvar::new(),
    });

    cursor_table.set("get", cursor_table_get)?;
    lua.globals().set("cursor", cursor_table)?;
    Ok(())
}
