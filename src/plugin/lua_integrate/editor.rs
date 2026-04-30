use mlua;
use crate::plugin::action::PluginAction;

fn get_char_at(x: usize, y: usize) -> Result<(), mlua::Error> {
  todo!("Implement getting char in editor; plugin/lua_integrate/editor.rs");
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

#[inline]
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

#[inline]
pub fn integrate_editor(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let editor_table = lua.create_table()?;

    let insert_table = lua.create_table()?;

    let tx_cursor = tx.clone();
    insert_table.set("cursor", lua.create_function(move |_, value: char| {
        let _ = tx_cursor.send(PluginAction::InsertCharAtCursor { txt: value });
        Ok(())
    })?)?;

    insert_char_at_cursor(lua, tx, &insert_table)?;

    //let tx_set = tx.clone();
    // insert_table.set("set", lua.create_function(move |_, (c, x, y): (char, usize, usize)| {
    //     Ok(())
    // })?)?;

    // //let tx_get = tx.clone();
    // insert_table.set("get", lua.create_function(move |_, (c, x, y): (char, usize, usize)| {
    //     Ok(())
    // })?)?;

    editor_table.set("insert", insert_table)?;

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
