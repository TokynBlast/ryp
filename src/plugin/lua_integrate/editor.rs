use compact_str::CompactString;
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{CharResponder, PluginAction, StrResponder};

// editor.get.char_at(x: usize, y: usize)
fn get_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, get_table: &mlua::Table) -> Result<(), mlua::Error> {
    let responder = Arc::new(CharResponder {
        c: Mutex::new(None),
        signal: Condvar::new(),
    });
    let responder_clone = responder.clone();
    let tx = tx.clone();

    get_table.set("char_at",
        lua.create_function(move |lua, (x, y): (usize, usize)| {
            let _ = tx.send(PluginAction::GetCharAt { x, y, responder: responder_clone.clone() });
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

// editor.get.line(line: usize)
fn get_line(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, get_table: &mlua::Table) -> Result<(), mlua::Error> {
    let responder = Arc::new(StrResponder {
        string: Mutex::new(Some(CompactString::default())),
        signal: Condvar::new(),
    });

    let responder_clone = responder.clone();
    let tx = tx.clone();

    get_table.set("line",
        lua.create_function(move |lua, line: usize| {
            let _ = tx.send(PluginAction::GetLine { line, responder: responder_clone.clone() });
            let mut lock = responder_clone.string.lock();
            if lock.is_none() {
                responder_clone.signal.wait(&mut lock);
            }
            let info = lock.take().map(|s| s.to_string());
            lua.to_value(&info)
        })?
    )?;
    Ok(())
}

// editor.set.char(x: usize, y: usize, c: char)
fn set_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, set_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    set_table.set("char",
        lua.create_function(move |_lua, (x, y, c) : (usize, usize, char)| {
              let _ = tx.send(PluginAction::SetChar { x, y, c });
              Ok(())
          })?
      )?;
      Ok(())
}

fn insert_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, insert_table: &mlua::Table) -> Result<(), mlua::Error> {
    todo!("Implement inserting a char at specific place; plugin/lua_integrate/editor.rs")
}

// editor.insert.cursor(txt: char)
fn insert_char_at_cursor(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, insert_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    insert_table.set("cursor",
        lua.create_function(move |_lua, txt: char| {
            let _ = tx.send(PluginAction::InsertCharAtCursor { txt });
            Ok(())
        })?
    )?;
    Ok(())
}

pub fn integrate_editor(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let editor_table = lua.create_table()?;
    let insert_table = lua.create_table()?;
    let get_table = lua.create_table()?;
    let set_table = lua.create_table()?;

    insert_char_at_cursor(lua, tx, &insert_table)?;
    get_char_at(lua, tx, &get_table)?;
    set_char_at(lua, tx, &set_table)?;
    get_line(lua, tx, &get_table)?;

    editor_table.set("insert", insert_table)?;
    editor_table.set("get", get_table)?;
    editor_table.set("set", set_table)?;

    let proxy = lua.create_table()?;
    let metatable = lua.create_table()?;

    let internal_editor = editor_table.clone();
    metatable.set("__index", lua.create_function(move |_, (_, key): (mlua::Value, String)| {
        internal_editor.get::<mlua::Value>(key)
    })?)?;

    proxy.set_metatable(Some(metatable))?;

    lua.globals().set("editor", proxy)?;

    Ok(())
}
