pub enum PluginAction {
  SetSetting { name: String, value: mlua::Value },
  InsertText { text: String, x: usize, y: usize },
  GetAppInfo { respond_to: oneshot::Sender<String> },
}
