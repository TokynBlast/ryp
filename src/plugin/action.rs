use parking_lot::{Mutex, Condvar};
use triomphe::Arc;

#[derive(Debug)]
pub struct SerdeResponder {
    pub value: Mutex<Option<serde_json::Value>>,
    pub signal: Condvar,
}

#[derive(Debug)]
pub struct UsizeResponder {
    pub number: Mutex<Option<usize>>,
    pub signal: Condvar
}
#[derive(Debug)]
pub struct UsizeVecResponder {
    pub numbers: Mutex<Option<Vec<usize>>>,
    pub signal: Condvar
}

#[derive(Debug)]
pub struct StrResponder {
    pub string: Mutex<Option<String>>,
    pub signal: Condvar
}

#[derive(Debug)]
pub struct CharResponder {
    pub c: Mutex<Option<char>>,
    pub signal: Condvar,
}

#[derive(Debug, Clone)]
pub enum PluginAction {
    MakeSetting { name: String, value: serde_json::Value },
    GetSettingValue { name: String, responder: Arc<SerdeResponder> },
    DebugLog { message: String },
    SetSetting { name: String, value: serde_json::Value },
    InsertStrAtCursor { txt: String },
    GetKeyPress { responder: Arc<StrResponder> },
    GetCursorPos { responder: Arc<UsizeVecResponder> },
    GetCursorX { responder: Arc<UsizeResponder> },
    GetCursorY { responder: Arc<UsizeResponder> },
    SetCursorX { x: usize },
    SetCursorY { y: usize },
    SetCursorPos { pos: Vec<usize> },
    GetLine { line: usize, responder: Arc<StrResponder> },
    SetLine { line: usize, contents: String },
    SetStrAt { from: Vec<usize>, to: Vec<usize>, txt: String },
    GetStrAt { from: Vec<usize>, to: Vec<usize>, responder: Arc<StrResponder> },
    InsertStrAt { pos: Vec<usize>,txt: String },
    SetCharAt { pos: Vec<usize>, c: char },
    SetCharAtCursor { c: char },
    GetCharAt { pos: Vec<usize>, responder: Arc<CharResponder> },
    GetCharAtCursor { responder: Arc<CharResponder> },
    InsertCharAt { pos: Vec<usize>, c: char },
    InsertCharAtCursor { c: char },
    MakeCommand { name: String, func: mlua::Function },
    RemoveCommand { name: String },
}
