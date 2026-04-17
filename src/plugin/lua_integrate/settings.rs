use mlua;
use crate::plugin::{action::PluginAction};
use mlua::LuaSerdeExt;

#[inline]
fn add_setting(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error>  {
    // We clone the sender for each function
    let tx_add = tx.clone();

    settings_table.set("add",
        lua.create_function(move |lua, (name, value): (String, mlua::Value)| {
            let json_value: serde_json::Value = lua.from_value(value)?;
            let _ = tx_add.send(PluginAction::MakeSetting { name, value: json_value }).ok();
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

            let (resp_tx, resp_rx) = crossbeam::channel::bounded(1);

            // Send request for value
            let _ = tx_get.send(PluginAction::GetSettingValue { name, tx_respond: resp_tx });

            // Wait for value
            let info = resp_rx.recv()
                .map_err(|_| mlua::Error::RuntimeError(format!("App died while getting {}", name_on_error)))?;

            lua.to_value(&info)
        })?
    )?;
    Ok(())
}

#[inline]
fn set_setting_value(lua: &mlua::Lua, tx: &crossbeam::channel::Sender<PluginAction>, settings_table: &mlua::Table) -> Result<(), mlua::Error> {
    let tx_set = tx.clone();

    settings_table.set("set",
        lua.create_function(move |lua, (name, value): (String, mlua::Value)| {
            let json_value: serde_json::Value = lua.from_value(value)?;
            let _ = tx_set.send(PluginAction::SetSetting { name, value: json_value });
            Ok(())
        })?
    )?;
    Ok(())
}

#[inline]
pub fn integrate_settings(lua: &mlua::Lua, tx: crossbeam::channel::Sender<PluginAction>) -> Result<(), mlua::Error> {
    let settings_table = lua.create_table()?;
    let globals = lua.globals();
    add_setting(lua, &tx, &settings_table)?;
    get_setting_value(lua, &tx, &settings_table)?;
    set_setting_value(lua, &tx, &settings_table)?;
    globals.set("settings", settings_table)?;
    Ok(())
}
