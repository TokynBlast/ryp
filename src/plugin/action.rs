use parking_lot::{Mutex, Condvar};
use triomphe::Arc;
use compact_str::CompactString;

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
    pub string: Mutex<Option<CompactString>>,
    pub signal: Condvar
}

#[derive(Debug, Clone)]
pub enum PluginAction {
    MakeSetting { name: String, value: serde_json::Value },
    GetSettingValue { name: String, responder: Arc<SerdeResponder> },
    DebugLog { message: CompactString },
    SetSetting { name: String, value: serde_json::Value },
    InsertStrAtCursor { txt: CompactString },
    GetKeyPress { responder: Arc<StrResponder> },
    GetCursorPos { responder: Arc<UsizeVecResponder> },
    GetCursorX { responder: Arc<UsizeResponder> },
    GetCursorY { responder: Arc<UsizeResponder> },
    SetCursorX { x: usize },
    SetCursorY { y: usize },
    SetCursorPos { pos: Vec<usize> },
    GetLine { line: usize, responder: Arc<StrResponder> },
    SetLine { line: usize, contents: CompactString },
    SetStrAt { from: Vec<usize>, to: Vec<usize>, txt: CompactString },
    GetStrAt { from: Vec<usize>, to: Vec<usize>, responder: Arc<StrResponder> },
    InsertStrAt { pos: Vec<usize>, txt: CompactString }
}
