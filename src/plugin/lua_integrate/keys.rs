use mlua::{Lua, Value, Function, RegistryKey, Result, Error};
use crossterm::event::{KeyCode, KeyModifiers};

// TODO: This needs to simply tap into the editor, rather than being seperate...
//       Rather than _ do nothing, we send the stuff to here, where it sits, then it gets cleared later :3

struct KeyPress {
    keys: Vec<KeyCode>,
    mods: KeyModifiers,
}

// wiring to expose register_shortcut to Lua
fn setup(lua: &Lua) -> Result<()> {
    todo!();
}
