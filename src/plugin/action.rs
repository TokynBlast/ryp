pub enum PluginAction {
  MakeSetting { name: String, value: mlua::Value },
  InsertText { text: String, x: usize, y: usize },
  GetSettingValue { name: String, tx_respond: oneshot::Sender<serde_json::Value> },
  DebugLog { message: String },
  SetSetting { name: String, value: mlua::Value },
}
