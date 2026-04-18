use parking_lot::{Mutex, Condvar};
use triomphe::Arc;

pub struct Responder {
    pub value: Mutex<Option<serde_json::Value>>,
    pub signal: Condvar,
}

pub enum PluginAction {
  MakeSetting { name: String, value: serde_json::Value },
  InsertText { text: char },
  GetSettingValue { name: String, responder: Arc<Responder> },
  DebugLog { message: String },
  SetSetting { name: String, value: serde_json::Value },
}
