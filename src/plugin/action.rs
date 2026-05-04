use parking_lot::{Mutex, Condvar};
use triomphe::Arc;

#[derive(Debug)]
pub struct SettingResponder {
    pub value: Mutex<Option<serde_json::Value>>,
    pub signal: Condvar,
}

#[derive(Debug, Clone)]
pub enum PluginAction {
  MakeSetting { name: String, value: serde_json::Value },
  GetSettingValue { name: String, responder: Arc<SettingResponder> },
  DebugLog { message: String },
  SetSetting { name: String, value: serde_json::Value },
  InsertCharAtCursor { txt: char },
}
