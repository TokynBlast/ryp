use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use mlua::{self, LuaSerdeExt};
use crate::plugin::action::{PluginAction, CharResponder};

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

fn get_line_at(line: usize) -> Result<(), mlua::Error> {
    todo!("Implement getting a line in editor; plugin/lua_integrate/editor.rs")
}

fn set_char_at(char_to_place: char, x: usize, y: usize) -> Result<(), mlua::Error> {
    todo!("Implement setting a char at specific place; plugin/lua_integrate/editor.rs")
}

fn set_char_on_line(char_to_place: char, line: usize, y: usize) -> Result<(), mlua::Error> {
    todo!("Implement setting a char on a specific line; plugin/lua_integrate/editor.rs")
}

fn insert_char_at(char_to_place: char, x: usize, y: usize) -> Result<(), mlua::Error> {
    todo!("Implement inserting a char at specific place; plugin/lua_integrate/editor.rs")
}

fn insert_char_on_line(char_to_place: char, line: usize, y: usize) -> Result<(), mlua::Error> {
    todo!("Implement inserting a char on a specific line; plugin/lua_integrate/editor.rs")
}

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
