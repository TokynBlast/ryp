use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use compact_str::CompactString;

#[derive(Debug)]
pub struct SerdeResponder {
    pub value: Mutex<Option<serde_json::Value>>,
    pub signal: Condvar,
}

#[derive(Debug)]
pub struct CharResponder {
    pub c: Mutex<Option<char>>,
    pub signal: Condvar,
}

#[derive(Debug)]
pub struct UsizeResponder {
    pub number: Mutex<usize>,
    pub signal: Condvar
}

#[derive(Debug)]
pub struct StrResponder {
    pub string: Mutex<Option<CompactString>>,
    pub signal: Condvar
}

#[derive(Debug, Clone)]
pub enum PluginAction {
    MakeSetting { name: String, value: serde_json::Value },
    GetSettingValue { name: String, responder: Arc<SerdeResponder> },
    DebugLog { message: String },
    SetSetting { name: String, value: serde_json::Value },
    InsertCharAtCursor { txt: char },
    GetKeyPress { responder: Arc<CharResponder> },
    GetCharAt { x: usize, y: usize, responder: Arc<CharResponder> },
    GetCursorX { responder: Arc<UsizeResponder> },
    GetCursorY { responder: Arc<UsizeResponder> },
    SetCursorX { x: usize },
    SetCursorY { y: usize },
    SetCursorPos { x: usize, y: usize },
    GetLine { line: usize, responder: Arc<StrResponder> },
    SetLine { line: usize, contents: CompactString },
    SetChar { x: usize, y: usize, c: char },
}
