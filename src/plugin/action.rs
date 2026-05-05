use parking_lot::{Mutex, Condvar};
use triomphe::Arc;

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

#[derive(Debug, Clone)]
pub enum PluginAction {
    MakeSetting { name: String, value: serde_json::Value },
    GetSettingValue { name: String, responder: Arc<SerdeResponder> },
    DebugLog { message: String },
    SetSetting { name: String, value: serde_json::Value },
    InsertCharAtCursor { txt: char },
    GetKeyPress { responder: Arc<CharResponder> },
    GetCharAt {x: usize, y: usize, responder: Arc<CharResponder> },
}
