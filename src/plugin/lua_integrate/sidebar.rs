// NOTE: May not be used
use crate::plugin::action::PluginAction;

fn add_sidebar_button(lua: &mlua::Lua, tx: &crossbeam_channel::Sender<PluginAction>, set_table: &mlua::Table) -> Result<(), mlua::Error> {
    todo!("Implement Lua sidebar adding and icon setting");
}
