use mlua;
use crate::plugin::{action::PluginAction};
use mlua::LuaSerdeExt;

#[inline]
fn add_setting(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error>  {
    // We clone the sender for each function
    let tx_add = tx.clone();

    settings_table.set("add",
        lua.create_function(move |_, (name, value): (String, mlua::Value)| {
            let _ = tx_add.send(PluginAction::MakeSetting { name, value });
            Ok(())
        })?
    )?;
    Ok(())
}

#[inline]
fn get_setting_value(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error>  {
    let tx_get = tx.clone();
    settings_table.set("get",
        lua.create_function(move |lua, name: String| {
            let name_on_error: String = name.clone();

            let (tx_respond, rx_respond) = oneshot::channel::<String>();
            // Send request for value
            let _ = tx_get.send(PluginAction::GetSettingValue { name, tx_respond });
            // Wait for value
            let info = rx_respond.try_recv().map_err(|_| mlua::Error::RuntimeError(format!("Fatal error: could not get value {}", name_on_error).into()))?;

            Ok(info)
        })?
    )?;
    Ok(())
}

#[inline]
fn set_setting_value(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx_set = tx.clone();

    settings_table.set("set",
        lua.create_function(move |_, (name, value): (String, mlua::Value)| {
            let _ = tx_set.send(PluginAction::SetSetting { name, value });
            Ok(())
        })?
    )?;
    Ok(())
}

#[inline]
pub fn integrate_settings(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let settings_table = lua.create_table()?;
    let globals = lua.globals();
    get_setting_value(lua, &tx, &settings_table)?;
    add_setting(lua, &tx, &settings_table)?;
    globals.set("settings", settings_table)?;
    Ok(())
}
