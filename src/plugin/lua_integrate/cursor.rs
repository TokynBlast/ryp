use crate::plugin::action::{PluginAction, UsizeResponder, UsizeVecResponder};
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};

// cursor.pos.get()
fn get_cursor_pos(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_get: &mlua::Table, responder: Arc<UsizeVecResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table_get.set("get",
        lua.create_function(move |lua, ()|{
              let _ = tx.send(PluginAction::GetCursorPos { responder: responder_clone.clone() });
              let mut lock = responder_clone.numbers.lock();
              responder_clone.signal.wait(&mut lock);
              let info = lock.clone();
              lua.to_value(&info)
        })?
    )
}

// cursor.x.get()
fn get_cursor_x(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_get: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table_get.set("get",
        lua.create_function(move |lua, ()| {
            let _ = tx.send(PluginAction::GetCursorX { responder: responder_clone.clone() });
            let mut lock = responder_clone.number.lock();
            responder_clone.signal.wait(&mut lock);

            let info = *lock;
            lua.to_value(&info)
        })?
    )
}

// cursor.y.get()
fn get_cursor_y(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_get: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table_get.set("get",
        lua.create_function(move |lua, ()| {
            let _ = tx.send(PluginAction::GetCursorY { responder: responder_clone.clone() });
            let mut lock = responder_clone.number.lock();
            responder_clone.signal.wait(&mut lock);

            let info = *lock;
            lua.to_value(&info)
        })?
    )
}

//cursor.pos.set(pos: Vec<usize>)
fn set_cursor_pos(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_set: &mlua::Table)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    cursor_table_set.set("set",
        lua.create_function(move |_lua, pos: Vec<usize>| {
            let _ = tx.send(PluginAction::SetCursorPos { pos });
            Ok(())
        })?
    )
}

// cursor.x.set(x: usize)
fn set_cursor_x(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_set: &mlua::Table)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    cursor_table_set.set("set",
        lua.create_function(move |_lua, x: usize| {
            let _ = tx.send(PluginAction::SetCursorX { x });
            Ok(())
        })?
    )
}

// cursor.y.set(y: usize)
fn set_cursor_y(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table_set: &mlua::Table)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    cursor_table_set.set("set",
        lua.create_function(move |_lua, y: usize| {
            let _ = tx.send(PluginAction::SetCursorY { y });
            Ok(())
        })?
    )
}

pub fn integrate_cursor_pos(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let cursor_table = lua.create_table()?;
    let cursor_table_x = lua.create_table()?;
    let cursor_table_y = lua.create_table()?;
    let cursor_table_pos = lua.create_table()?;

    let responder = Arc::new(UsizeResponder {
        number: Mutex::new(0),
        signal: Condvar::new(),
    });

    let vec_responder = Arc::new(UsizeVecResponder {
        numbers: Mutex::new(vec![0, 0]),
        signal: Condvar::new(),
    });

    get_cursor_pos(lua, tx, &cursor_table_pos, vec_responder)?;
    get_cursor_x(lua, tx, &cursor_table_x, &responder)?;
    get_cursor_y(lua, tx, &cursor_table_y, &responder)?;

    set_cursor_pos(lua, tx, &cursor_table_pos)?;
    set_cursor_x(lua, tx, &cursor_table_x)?;
    set_cursor_y(lua, tx, &cursor_table_y)?;

    cursor_table.set("y", cursor_table_y)?;
    cursor_table.set("x", cursor_table_x)?;
    cursor_table.set("pos", cursor_table_pos)?;
    lua.globals().set("cursor", cursor_table)?;
    Ok(())
}
