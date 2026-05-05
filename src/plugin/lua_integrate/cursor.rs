use crate::plugin::action::{PluginAction, UsizeResponder};
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};

fn add_x(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
    let tx = tx.clone();
    let responder_clone = responder.clone();
    cursor_table.set("x",
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

fn add_y(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, cursor_table: &mlua::Table, responder: &Arc<UsizeResponder>)  -> Result<(), mlua::Error> {
  let tx = tx.clone();
  let responder_clone = responder.clone();
  cursor_table.set("y",
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

    let responder = Arc::new(UsizeResponder {
        number: Mutex::new(0),
        signal: Condvar::new(),
    });

    add_x(lua, tx, &cursor_table, &responder)?;
    add_y(lua, tx, &cursor_table, &responder)?;

    cursor_table.set("y", 0)?;
    lua.globals().set("cursor", cursor_table)?;
    Ok(())
}
