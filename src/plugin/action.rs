pub enum PluginAction {
  MakeSetting { name: String, value: mlua::Value },
  InsertText { text: String, x: usize, y: usize },
  DebugLog { message: String },
}
