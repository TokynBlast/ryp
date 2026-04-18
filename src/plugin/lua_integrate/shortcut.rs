use mlua::{Lua, Value, Function, RegistryKey, Result, Error};
use crossterm::event::{KeyCode, KeyModifiers};
use parking_lot::Mutex;
use triomphe::Arc;

struct Shortcut {
    keys: Vec<KeyCode>,
    mods: KeyModifiers,
    handler: RegistryKey,
}

struct Manager {
    lua: Arc<Lua>,
    shortcuts: Vec<Shortcut>,
    // runtime buffer of recent keys:
    seq_buf: Vec<KeyCode>,
}

impl Manager {
    fn new(lua: Arc<Lua>) -> Self {
        Self { lua, shortcuts: Vec::new(), seq_buf: Vec::new() }
    }

    fn register_from_lua(&mut self, keys_val: Value, mods_val: Value, func: Function) -> Result<()> {
        let keys = parse_keys(keys_val)?;
        let mods = parse_mods(mods_val)?;
        let key = self.lua.create_registry_value(func)?;
        self.shortcuts.push(Shortcut { keys, mods, handler: key });
        Ok(())
    }

    // called when a key event happens
    fn push_key_event(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<()> {
        self.seq_buf.push(code.clone());
        // trim buffer to longest registered sequence
        let max_len = self.shortcuts.iter().map(|s| s.keys.len()).max().unwrap_or(0);
        if self.seq_buf.len() > max_len { self.seq_buf.drain(0..(self.seq_buf.len()-max_len)); }

        // check shortcuts
        for sc in &self.shortcuts {
            if mods == sc.mods && self.seq_buf.ends_with(&sc.keys) {
                // call Lua handler
                let f: Function = self.lua.registry_value(&sc.handler)?;
                f.call::<()>(())?;
            }
        }
        Ok(())
    }
}

// helpers to parse keys/mods from Lua input
fn parse_keys(val: Value) -> Result<Vec<KeyCode>> {
    match val {
        Value::Table(t) => {
            let mut out = Vec::new();
            for pair in t.sequence_values::<String>() {
                let s = pair.map_err(Error::external)?;
                out.push(string_to_keycode(&s).ok_or_else(|| Error::external("invalid key"))?);
            }
            Ok(out)
        }
        _ => Err(Error::external("keys must be an array of strings")),
    }
}

fn parse_mods(val: Value) -> Result<KeyModifiers> {
    match val {
        Value::Table(t) => {
            let mut km = KeyModifiers::empty();
            for pair in t.sequence_values::<String>() {
                let s = pair.map_err(Error::external)?;
                match s.to_lowercase().as_str() {
                    "ctrl" | "control" => km.insert(KeyModifiers::CONTROL),
                    "shift" => km.insert(KeyModifiers::SHIFT),
                    "alt" => km.insert(KeyModifiers::ALT),
                    _ => return Err(Error::external("unknown modifier")),
                }
            }
            Ok(km)
        }
        _ => Err(Error::external("mods must be an array")),
    }
}

fn string_to_keycode(s: &str) -> Option<KeyCode> {
    if s.len() == 1 {
        Some(KeyCode::Char(s.chars().next().unwrap()))
    } else {
        match s.to_lowercase().as_str() {
            "enter" => Some(KeyCode::Enter),
            "left" => Some(KeyCode::Left),
            "right" => Some(KeyCode::Right),
            "up" => Some(KeyCode::Up),
            "down" => Some(KeyCode::Down),
            "tab" => Some(KeyCode::Tab),
            "esc" | "escape" => Some(KeyCode::Esc),
            "del" | "delete" => Some(KeyCode::Delete),
            "back" | "backspace" => Some(KeyCode::Backspace),
            "f1" => Some(KeyCode::F(1)),
            "f2" => Some(KeyCode::F(2)),
            "f3" => Some(KeyCode::F(3)),
            "f4" => Some(KeyCode::F(4)),
            "f5" => Some(KeyCode::F(5)),
            "f6" => Some(KeyCode::F(6)),
            "f7" => Some(KeyCode::F(7)),
            "f8" => Some(KeyCode::F(8)),
            "f9" => Some(KeyCode::F(9)),
            "f10" => Some(KeyCode::F(10)),
            "f11" => Some(KeyCode::F(11)),
            "f12" => Some(KeyCode::F(12)),
            _ => None,
        }
    }
}

// wiring to expose register_shortcut to Lua
fn setup(lua: &Lua, mgr: Arc<Mutex<Manager>>) -> Result<()> {
    let mgr_clone = mgr.clone();
    let reg = lua.create_function(move |_, (keys, mods, func): (Value, Value, Function)| {
        let mut m = mgr_clone.lock();
        m.register_from_lua(keys, mods, func)?;
        Ok(())
    })?;
    lua.globals().set("register_shortcut", reg)?;
    Ok(())
}
