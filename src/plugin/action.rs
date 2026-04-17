pub enum PluginAction {
  MakeSetting { name: String, value: serde_json::Value },
  InsertText { text: String, x: usize, y: usize },
  GetSettingValue { name: String, tx_respond: crossbeam::channel::Sender<serde_json::Value> },
  DebugLog { message: String },
  SetSetting { name: String, value: serde_json::Value },
}
