use compact_str::CompactString;
use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, StrResponder};

// editor.get.at(x: usize, y: usize)
fn get_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, get_table: &mlua::Table) -> Result<(), mlua::Error> {
    let responder = Arc::new(StrResponder {
        string: Mutex::new(None),
        signal: Condvar::new(),
    });
    let responder_clone = responder.clone();
    let tx = tx.clone();

    get_table.set("at",
        lua.create_function(move |lua, (from, to): (Vec<usize>, Vec<usize>)| {
            let _ = tx.send(PluginAction::GetStrAt { from, to, responder: responder_clone.clone() });
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
        })?
    )
}

// editor.get.line(line: usize)
fn get_line(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, get_table: &mlua::Table) -> Result<(), mlua::Error> {
    let responder = Arc::new(StrResponder {
        string: Mutex::new(None),
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

            let info = lock.clone();
            if info.is_some() {
                lua.to_value(&info.unwrap().to_string())
            } else {
                lua.to_value(&mlua::Nil)
            }
        })?
    )
}

// editor.set.char(pos: Vec<usize>, c: char)
fn set_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, set_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    set_table.set("char",
        lua.create_function(move |_lua, (pos, c) : (Vec<usize>, char)| {
              let _ = tx.send(PluginAction::SetCharAt { pos, c });
              Ok(())
          })?
    )
}

// editor.insert.char(pos: vec<usize>, txt: char)
fn insert_char_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, insert_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    insert_table.set("char",
        lua.create_function(move |_lua, (pos, txt) : (Vec<usize>, String)| {
            let _ = tx.send(PluginAction::InsertStrAt { pos, txt: CompactString::from(txt) });
            Ok(())
        })?
    )
}

// editor.insert.cursor(txt: char)
fn insert_char_at_cursor(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, insert_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    insert_table.set("cursor",
        lua.create_function(move |_lua, txt: String| {
            let _ = tx.send(PluginAction::InsertStrAtCursor { txt: CompactString::from(txt) });
            Ok(())
        })?
    )
}

// editor.insert.str(pos: Vec<usize>)
fn insert_str_at(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, insert_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx = tx.clone();
    insert_table.set("str",
        lua.create_function(move |_lua, txt: String| {
            let _ = tx.send(PluginAction::InsertStrAtCursor { txt: CompactString::from(txt) });
            Ok(())
        })?
    )
}

pub fn integrate_editor(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let editor_table = lua.create_table()?;
    let insert_table = lua.create_table()?;
    let get_table = lua.create_table()?;
    let set_table = lua.create_table()?;

    insert_char_at_cursor(lua, tx, &insert_table)?;
    insert_char_at(lua, tx, &insert_table)?;
    insert_str_at(lua, tx, &insert_table)?;

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
